use crate::errors::ScraperError;
use chrono::{DateTime, Utc};
use derive_new::new;
use log::{error, warn};
use regex::Regex;
use reqwest::get;
use scraper::{Html, Selector};
use serde_json::Value;

/// The primary data structure containing the metadata of a book.
#[derive(Debug, new, PartialEq)]
pub struct BookMetadata {
    /// The main title of the book.
    pub title: String,
    /// An optional subtitle of the book.
    pub subtitle: Option<String>,
    /// An optional description or summary of the book.
    pub description: Option<String>,
    /// The publisher of the book, if available.
    pub publisher: Option<String>,
    /// The publication date of the book, represented as a UTC datetime.
    pub publication_date: Option<DateTime<Utc>>,
    /// The ISBN of the book, if available.
    pub isbn: Option<String>,
    /// A list of contributors to the book, each represented as a `BookContributor`.
    pub contributors: Vec<BookContributor>,
    /// A list of genres associated with the book.
    pub genres: Vec<String>,
    /// The series information, if the book is part of a series, represented as a `BookSeries`.
    pub series: Option<BookSeries>,
    /// The number of pages in the book, if available.
    pub page_count: Option<i64>,
    /// The language of the book, if available.
    pub language: Option<String>,
    /// A URL to an image of the book's cover, if available.
    pub image_url: Option<String>,
}

/// Represents an individual who contributed to the book, such as an author or editor.
#[derive(Debug, new, PartialEq)]
pub struct BookContributor {
    /// The name of the contributor.
    pub name: String,
    /// The role of the contributor, such as "Author" or "Illustrator".
    pub role: String,
}

/// Represents series information for a book, including the series title and book's position within the series.
#[derive(Debug, new, PartialEq)]
pub struct BookSeries {
    /// The title of the series.
    pub title: String,
    /// The position of the book within the series, represented as a float to accommodate cases like "1.5".
    pub number: f32,
}

pub async fn fetch_metadata(goodreads_id: &str) -> Result<BookMetadata, ScraperError> {
    let metadata = extract_book_metadata(goodreads_id).await?;
    let amazon_id = extract_amazon_id(&metadata, goodreads_id)?;

    let (title, subtitle) = extract_title_and_subtitle(&metadata, &amazon_id)?;
    let description = extract_description(&metadata, &amazon_id);
    let image_url = extract_image_url(&metadata, &amazon_id);
    let contributors = extract_contributors(&metadata, &amazon_id);
    let genres = extract_genres(&metadata, &amazon_id);
    let publisher = extract_publisher(&metadata, &amazon_id);
    let publication_date = extract_publication_date(&metadata, &amazon_id);
    let isbn = extract_isbn(&metadata, &amazon_id);
    let page_count = extract_page_count(&metadata, &amazon_id);
    let language = extract_language(&metadata, &amazon_id);
    let series = extract_series(&metadata, &amazon_id);

    let metadata = BookMetadata::new(
        title,
        subtitle,
        description,
        publisher,
        publication_date,
        isbn,
        contributors,
        genres,
        series,
        page_count,
        language,
        image_url,
    );

    Ok(metadata)
}

async fn extract_book_metadata(goodreads_id: &str) -> Result<Value, ScraperError> {
    let url = format!("https://www.goodreads.com/book/show/{goodreads_id}");
    let document = Html::parse_document(&get(&url).await?.text().await?);
    let metadata_selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#)?;
    let metadata = &document.select(&metadata_selector).next();

    let metadata = match metadata {
        None => {
            error!("Failed to scrape book metadata");
            return Err(ScraperError::ScrapeError(
                "Failed to scrape book metadata".to_string(),
            ));
        }
        Some(m) => serde_json::from_str(&m.text().collect::<String>())?,
    };

    Ok(metadata)
}

fn extract_amazon_id(metadata: &Value, goodreads_id: &str) -> Result<String, ScraperError> {
    let amazon_id_key = format!("getBookByLegacyId({{\"legacyId\":\"{goodreads_id}\"}})");
    let amazon_id =
        &metadata["props"]["pageProps"]["apolloState"]["ROOT_QUERY"][amazon_id_key]["__ref"];
    let Some(amazon_id) = to_string(amazon_id) else {
        error!("Failed to scrape Amazon ID");
        return Err(ScraperError::ScrapeError(
            "Failed to scrape Amazon ID".to_string(),
        ));
    };

    Ok(amazon_id)
}

fn extract_title_and_subtitle(
    metadata: &Value,
    amazon_id: &str,
) -> Result<(String, Option<String>), ScraperError> {
    let title = &metadata["props"]["pageProps"]["apolloState"][amazon_id]["title"];
    let Some(title) = to_string(title) else {
        error!("Failed to scrape book title");
        return Err(ScraperError::ScrapeError(
            "Failed to scrape book title".to_string(),
        ));
    };

    match title.split_once(':') {
        Some((title, subtitle)) => Ok((title.to_string(), Some(subtitle.trim().to_string()))),
        None => Ok((title.to_string(), None)),
    }
}

fn extract_description(metadata: &Value, amazon_id: &str) -> Option<String> {
    let description = &metadata["props"]["pageProps"]["apolloState"][amazon_id]["description"];
    to_string(description)
}

fn extract_image_url(metadata: &Value, amazon_id: &str) -> Option<String> {
    let url = &metadata["props"]["pageProps"]["apolloState"][amazon_id]["imageUrl"];
    to_string(url)
}

fn extract_contributors(metadata: &Value, amazon_id: &str) -> Vec<BookContributor> {
    let mut contributors = Vec::new();

    let primary =
        metadata["props"]["pageProps"]["apolloState"][amazon_id]["primaryContributorEdge"]
            .as_object()
            .map(|obj| (to_string(&obj["role"]), to_string(&obj["node"]["__ref"])));

    match primary {
        Some((Some(role), Some(reference))) => {
            if let Some(contributor) = fetch_contributor(metadata, (role, reference)) {
                contributors.push(contributor);
            }
        }
        Some(_) => {
            warn!("Failed to parse contributor");
        }
        None => (),
    }

    let Some(secondary) =
        metadata["props"]["pageProps"]["apolloState"][amazon_id]["secondaryContributorEdges"]
            .as_array()
    else {
        return contributors
            .into_iter()
            .filter(|s| !s.name.to_lowercase().eq("unknown author"))
            .collect();
    };

    for contributor in secondary {
        let role = to_string(&contributor["role"]);
        let key = to_string(&contributor["node"]["__ref"]);
        if role.is_none() || key.is_none() {
            warn!("Failed to parse contributor");
            continue;
        }

        if let Some(contributor) = fetch_contributor(metadata, (role.unwrap(), key.unwrap())) {
            contributors.push(contributor);
        }
    }

    contributors
        .into_iter()
        .filter(|s| !s.name.to_lowercase().eq("unknown author"))
        .collect()
}

fn fetch_contributor(metadata: &Value, (role, key): (String, String)) -> Option<BookContributor> {
    let contributor = &metadata["props"]["pageProps"]["apolloState"][key]["name"];
    let name = to_string(contributor);
    if name.is_none() {
        warn!("Failed to parse contributor");
    }

    name.map(|n| BookContributor::new(n, role))
}

fn extract_genres(metadata: &Value, amazon_id: &str) -> Vec<String> {
    let genres = metadata["props"]["pageProps"]["apolloState"][amazon_id]["bookGenres"].as_array();

    let Some(genres) = genres else {
        return vec![];
    };

    genres
        .iter()
        .filter_map(|genre| {
            to_string(&genre["genre"]["name"]).or_else(|| {
                warn!("Failed to parse genre name");
                None
            })
        })
        .collect()
}

fn extract_publisher(metadata: &Value, amazon_id: &str) -> Option<String> {
    let publisher =
        &metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["publisher"];
    to_string(publisher)
}

fn extract_publication_date(metadata: &Value, amazon_id: &str) -> Option<DateTime<Utc>> {
    match &metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["publicationTime"] {
        Value::Null => None,
        Value::Number(number) => {
            let timestamp = number.as_i64().map(DateTime::from_timestamp_millis);

            if timestamp.is_none() {
                warn!("Failed to parse publication date");
            }

            timestamp.flatten()
        }
        _ => panic!("Publication date must be a timestamp"),
    }
}

fn extract_isbn(metadata: &Value, amazon_id: &str) -> Option<String> {
    let isbn = &metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["isbn"];
    if let Some(i) = to_string(isbn) {
        return Some(i);
    }

    let isbn13 = &metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["isbn13"];
    if let Some(i) = to_string(isbn13) {
        return Some(i);
    }

    let asin = &metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["asin"];
    to_string(asin)
}

fn extract_page_count(metadata: &Value, amazon_id: &str) -> Option<i64> {
    let count =
        metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["numPages"].as_i64();
    match count {
        Some(0) => None,
        c => c,
    }
}

fn extract_language(metadata: &Value, amazon_id: &str) -> Option<String> {
    let language =
        &metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["language"]["name"];
    to_string(language)
}

fn extract_series(metadata: &Value, amazon_id: &str) -> Option<BookSeries> {
    let series_array =
        metadata["props"]["pageProps"]["apolloState"][amazon_id]["bookSeries"].as_array()?;

    let series = series_array.first()?;

    let Some(position) = series["userPosition"]
        .as_str()
        .map(|s| s.split('-').next().unwrap_or(""))
        .and_then(|s| s.parse::<f32>().ok())
    else {
        warn!("Failed to parse series number");
        return None;
    };

    let Some(key) = to_string(&series["series"]["__ref"]) else {
        warn!("Failed to parse series key");
        return None;
    };

    let title = &metadata["props"]["pageProps"]["apolloState"][key]["title"];
    let Some(title) = to_string(title) else {
        warn!("Failed to parse series title");
        return None;
    };

    Some(BookSeries::new(title, position))
}

fn to_string(value: &Value) -> Option<String> {
    let re = Regex::new(r"\s{2,}").expect("Regex must be valid");
    value
        .as_str()
        .map(str::trim)
        .map(|s| re.replace_all(s, " ").to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_metadata_test() {
        let expected_series = Some(BookSeries::new(
            "Percy Jackson and the Olympians".to_string(),
            5.0,
        ));
        let expected_contributors = vec![BookContributor::new(
            "Rick Riordan".to_string(),
            "Author".to_string(),
        )];
        let expected_genres = vec![
            "Fantasy".to_string(),
            "Young Adult".to_string(),
            "Mythology".to_string(),
            "Fiction".to_string(),
            "Percy Jackson".to_string(),
            "Middle Grade".to_string(),
            "Adventure".to_string(),
            "Greek Mythology".to_string(),
            "Urban Fantasy".to_string(),
            "Childrens".to_string(),
        ];
        let expected_metadata = BookMetadata::new(
            "The Last Olympian".to_string(),
            None,
            Some("All year the half-bloods have been preparing for battle against the Titans, knowing the odds of victory are grim. \
            Kronos's army is stronger than ever, and with every god and half-blood he recruits, the evil Titan's power only grows.\
            <br /><br />While the Olympians struggle to contain the rampaging monster Typhon, Kronos begins his advance on New York City, \
            where Mount Olympus stands virtually unguarded. Now it's up to Percy Jackson and an army of young demigods to stop the Lord of Time. \
            <br /><br />In this momentous final book in the <i>New York Times</i> best-selling series, the long-awaited prophecy surrounding \
            Percy's sixteenth birthday unfolds. And as the battle for Western civilization rages on the streets of Manhattan, Percy faces a \
            terrifying suspicion that he may be fighting against his own fate.".to_string()),
            Some("Disney-Hyperion Books".to_string()),
            Some(DateTime::parse_from_rfc3339("2009-05-05T07:00:00Z").unwrap().to_utc()),
            Some("1423101472".to_string()),
            expected_contributors,
            expected_genres,
            expected_series,
            Some(381),
            Some("English".to_string()),
            Some("https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1723393514i/4556058.jpg".to_string()),
        );

        let metadata = fetch_metadata("4556058").await.unwrap();
        assert_eq!(metadata, expected_metadata);
    }
}
