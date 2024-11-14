use crate::errors::ScraperError;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde_json::Value;
use urlencoding::encode;

pub fn verify_id_exists(id: &str) -> bool {
    let url = format!("https://www.goodreads.com/book/show/{}", id);
    let response = get(&url).expect("Failed to fetch book page");
    response.status().is_success()
}

pub fn fetch_id_from_isbn(isbn: &str) -> Result<Option<String>, ScraperError> {
    let url = format!("https://www.goodreads.com/search?q={}", encode(isbn));
    let document = Html::parse_document(&get(&url)?.text()?);

    let metadata_selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#)?;

    let metadata = match document.select(&metadata_selector).next() {
        Some(metadata) => &metadata.text().collect::<String>(),
        None => return Ok(None),
    };

    let metadata: Value = serde_json::from_str(metadata)?;

    let goodreads_id = metadata["props"]["pageProps"]["params"]["book_id"]
        .as_str()
        .expect("Failed to extract Goodreads ID from ISBN search results");

    let goodreads_id = goodreads_id
        .chars()
        .take_while(|c| c.is_numeric())
        .collect::<String>();

    Ok(Some(goodreads_id))
}

pub fn fetch_id_from_title(title: &str) -> Result<Option<String>, ScraperError> {
    let results = search_books(title)?;

    for (found_title, _, found_id) in results {
        if matches(&found_title, title) {
            return Ok(Some(found_id));
        }
    }

    Ok(None)
}

pub fn fetch_id_from_title_and_author(
    title: &str,
    author: &str,
) -> Result<Option<String>, ScraperError> {
    let results = search_books(title)?;

    for (found_title, found_author, found_id) in results {
        if matches(&found_title, title) && matches(&found_author, author) {
            return Ok(Some(found_id));
        }
    }

    let results = search_books(&format!("{} {}", title, author))?;

    for (found_title, found_author, found_id) in results {
        if matches(&found_title, title) && matches(&found_author, author) {
            return Ok(Some(found_id));
        }
    }

    Ok(None)
}

fn search_books(query: &str) -> Result<Vec<(String, String, String)>, ScraperError> {
    let url = format!("https://www.goodreads.com/search?q={}", encode(query));

    let document = Html::parse_document(&get(&url)?.text()?);
    let title_selector = Selector::parse(r#"a[class="bookTitle"]"#)?;
    let author_selector = Selector::parse(r#"a[class="authorName"]"#)?;

    let mut results = Vec::new();

    for (title, author) in document
        .select(&title_selector)
        .zip(document.select(&author_selector))
    {
        let found_title = title.text().collect::<String>();
        let found_author = author.text().collect::<String>();
        let found_link = title
            .value()
            .attr("href")
            .expect("Failed to extract link from search result");
        let found_id = extract_goodreads_id(found_link);

        results.push((found_title, found_author, found_id));
    }
    Ok(results)
}

fn matches(str1: &str, str2: &str) -> bool {
    let str1 = str1
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let str2 = str2
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();

    str1.to_lowercase().contains(&str2.to_lowercase())
}

fn extract_goodreads_id(url: &str) -> String {
    url.splitn(4, '/')
        .nth(3)
        .expect("Failed to extract Goodreads ID")
        .split('?')
        .next()
        .expect("Failed to extract Goodreads ID")
        .chars()
        .take_while(|c| c.is_numeric())
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_id_from_title_test() {
        let book_title = "The Last Magician";
        assert_eq!(
            fetch_id_from_title(book_title).unwrap(),
            Some("30312855".to_string())
        );
    }

    #[test]
    fn fetch_id_from_title_not_found_test() {
        let book_title = "thistitledoesnotexist";
        assert_eq!(fetch_id_from_title(book_title).unwrap(), None);
    }

    #[test]
    fn fetch_id_from_title_and_author_test() {
        let book_title = "Fire";
        let book_author = "Kristin Cashore";
        assert_eq!(
            fetch_id_from_title_and_author(book_title, book_author).unwrap(),
            Some("6137154".to_string())
        );
    }

    #[test]
    fn fetch_id_from_title_and_author_not_found_test() {
        let book_title = "thistitledoesnotexist";
        let book_author = "noauthor";
        assert_eq!(
            fetch_id_from_title_and_author(book_title, book_author).unwrap(),
            None
        );
    }

    #[test]
    fn fetch_id_from_isbn_test() {
        let isbn = "9780063021426";
        assert_eq!(
            fetch_id_from_isbn(isbn).unwrap(),
            Some("57945316".to_string())
        )
    }

    #[test]
    fn fetch_id_from_isbn_not_found_test() {
        let isbn = "1234001592323";
        assert_eq!(fetch_id_from_isbn(isbn).unwrap(), None);
    }

    #[test]
    fn verify_id_exists_test() {
        let id = "57945316";
        assert_eq!(verify_id_exists(id), true);
    }

    #[test]
    fn verify_id_not_found_test() {
        let id = "bad_id";
        assert_eq!(verify_id_exists(id), false);
    }
}
