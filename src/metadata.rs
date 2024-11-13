use chrono::{DateTime, Utc};
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde_json::Value;

use crate::{
    models::{BookMetadata, BookSeries, Contributor, ScraperError},
    search::search_book,
};

pub fn search_metadata(
    book_title: &str,
    book_author: Option<&str>,
) -> Result<Option<BookMetadata>, ScraperError> {
    let goodreads_id = match search_book(book_title, book_author)? {
        Some(id) => id,
        None => return Ok(None),
    };
    let metadata = extract_metadata(&goodreads_id)?;
    let amazon_id = extract_amazon_id(&metadata, &goodreads_id);

    let (title, subtitle) = extract_title_and_subtitle(&metadata, &amazon_id);
    let description = extract_description(&metadata, &amazon_id);
    let image_url = extract_image_url(&metadata, &amazon_id);
    let contributors = extract_contributors(&metadata, &amazon_id);
    let genres = extract_genres(&metadata, &amazon_id);
    let publisher = extract_publisher(&metadata, &amazon_id);
    let publication_date = extract_publication_date(&metadata, &amazon_id);
    let isbn = extract_isbn(&metadata, &amazon_id);
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
        image_url,
    );

    Ok(Some(metadata))
}

fn extract_metadata(goodreads_id: &str) -> Result<Value, ScraperError> {
    let url = format!("https://www.goodreads.com/book/show/{}", goodreads_id);
    let document = Html::parse_document(&get(&url)?.text()?);
    let metadata_selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#)?;
    let metadata: Value = serde_json::from_str(
        &document
            .select(&metadata_selector)
            .next()
            .expect("Failed to find metadata script")
            .text()
            .collect::<String>(),
    )?;
    Ok(metadata)
}

fn extract_amazon_id(metadata: &Value, goodreads_id: &str) -> String {
    let amazon_id_key = format!("getBookByLegacyId({{\"legacyId\":\"{}\"}})", goodreads_id);
    metadata["props"]["pageProps"]["apolloState"]["ROOT_QUERY"][amazon_id_key]["__ref"]
        .as_str()
        .expect("Amazon ID must be a string")
        .to_string()
}

fn extract_title_and_subtitle(metadata: &Value, amazon_id: &str) -> (String, Option<String>) {
    let title = metadata["props"]["pageProps"]["apolloState"][amazon_id]["title"]
        .as_str()
        .expect("Title must be a string");

    match title.split_once(":") {
        Some((title, subtitle)) => (title.to_string(), Some(subtitle.trim().to_string())),
        None => (title.to_string(), None),
    }
}

fn extract_description(metadata: &Value, amazon_id: &str) -> Option<String> {
    metadata["props"]["pageProps"]["apolloState"][amazon_id]["description"]
        .as_str()
        .map(|description| description.to_string())
}

fn extract_image_url(metadata: &Value, amazon_id: &str) -> Option<String> {
    metadata["props"]["pageProps"]["apolloState"][amazon_id]["imageUrl"]
        .as_str()
        .map(|image_url| image_url.to_string())
}

fn extract_contributors(metadata: &Value, amazon_id: &str) -> Vec<Contributor> {
    let mut contributors = Vec::new();

    let primary = metadata["props"]["pageProps"]["apolloState"][amazon_id]
        ["primaryContributorEdge"]
        .as_object()
        .map(|obj| {
            (
                obj["role"]
                    .as_str()
                    .expect("Contributor role must be a string")
                    .to_string(),
                obj["node"]["__ref"]
                    .as_str()
                    .expect("Contributor key must be a string")
                    .to_string(),
            )
        })
        .expect("Primary contributor must be an object");

    contributors.push(fetch_contributor(metadata, primary));

    let secondary = metadata["props"]["pageProps"]["apolloState"][amazon_id]
        ["secondaryContributorEdges"]
        .as_array()
        .expect("Secondary contributors must be an array");

    for contributor in secondary {
        let role = contributor["role"]
            .as_str()
            .expect("Contributor role must be a string")
            .to_string();
        let key = contributor["node"]["__ref"]
            .as_str()
            .expect("Contributor key must be a string")
            .to_string();
        contributors.push(fetch_contributor(metadata, (role, key)));
    }

    contributors
}

fn fetch_contributor(metadata: &Value, (role, key): (String, String)) -> Contributor {
    let contributor = &metadata["props"]["pageProps"]["apolloState"][key];
    let name = contributor["name"]
        .as_str()
        .expect("Name must be a string")
        .to_string();
    Contributor::new(name, role)
}

fn extract_genres(metadata: &Value, amazon_id: &str) -> Vec<String> {
    metadata["props"]["pageProps"]["apolloState"][amazon_id]["bookGenres"]
        .as_array()
        .expect("Genres must be an array")
        .iter()
        .map(|genre| {
            genre["genre"]["name"]
                .as_str()
                .expect("Genre name must be a string")
                .to_string()
        })
        .collect()
}

fn extract_publisher(metadata: &Value, amazon_id: &str) -> Option<String> {
    metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["publisher"]
        .as_str()
        .map(|publisher| publisher.to_string())
}

fn extract_publication_date(metadata: &Value, amazon_id: &str) -> Option<DateTime<Utc>> {
    metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["publicationTime"]
        .as_i64()
        .map(|time| DateTime::from_timestamp_millis(time))
        .expect("Publication date must be present")
}

fn extract_isbn(metadata: &Value, amazon_id: &str) -> Option<String> {
    metadata["props"]["pageProps"]["apolloState"][amazon_id]["details"]["isbn"]
        .as_str()
        .map(|isbn| isbn.to_string())
}

fn extract_series(metadata: &Value, amazon_id: &str) -> Option<BookSeries> {
    let series_array = metadata["props"]["pageProps"]["apolloState"][amazon_id]["bookSeries"]
        .as_array()
        .expect("Book series must be an array");

    if let Some(series) = series_array.first() {
        let position = series["userPosition"]
            .as_str()
            .expect("Series position must be string")
            .parse::<f32>()
            .expect("Float parsing failed");
        let key = series["series"]["__ref"]
            .as_str()
            .expect("Series key must be string");
        let title = metadata["props"]["pageProps"]["apolloState"][key]["title"]
            .as_str()
            .expect("Series title must be string")
            .to_string();
        Some(BookSeries::new(title, position))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_metadata() {
        let title = "The Last Olympian";
        let author = None;

        let expected_series = Some(BookSeries::new(
            "Percy Jackson and the Olympians".to_string(),
            5.0,
        ));
        let expected_contributors = vec![Contributor::new(
            "Rick Riordan".to_string(),
            "Author".to_string(),
        )];
        let expected_genres = vec![
            "Fantasy".to_string(),
            "Young Adult".to_string(),
            "Mythology".to_string(),
            "Fiction".to_string(),
            "Middle Grade".to_string(),
            "Adventure".to_string(),
            "Greek Mythology".to_string(),
            "Urban Fantasy".to_string(),
            "Childrens".to_string(),
            "Audiobook".to_string(),
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
            Some("https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1723393514i/4556058.jpg".to_string()),
        );

        let metadata = search_metadata(title, author).unwrap();
        assert_eq!(metadata, Some(expected_metadata));
    }
}
