use std::str;

use crate::models::ScraperError;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use urlencoding::encode;

pub fn search_book(
    book_title: &str,
    book_author: Option<&str>,
) -> Result<Option<String>, ScraperError> {
    let result = match book_author {
        Some(author) => query_book(book_title, Some(author), &format!("{} {}", book_title, author))?,
        None => None,
    };

    Ok(result.or(query_book(book_title, book_author, book_title)?))
}

fn query_book(
    book_title: &str,
    book_author: Option<&str>,
    query: &str,
) -> Result<Option<String>, ScraperError> {
    let url = format!("https://www.goodreads.com/search?q={}", encode(query));

    let document = Html::parse_document(&get(&url)?.text()?);
    let title_selector = Selector::parse(r#"a[class="bookTitle"]"#)?;
    let author_selector = Selector::parse(r#"a[class="authorName"]"#)?;

    for (title, author) in document
        .select(&title_selector)
        .zip(document.select(&author_selector))
    {
        let found_title = &title.text().collect::<String>();
        let found_author = &author.text().collect::<String>();
        let found_link = title
            .value()
            .attr("href")
            .expect("Failed to extract link from search result");
        let found_id = extract_goodreads_id(&found_link)?;

        if matches(found_title, Some(book_title)) && matches(found_author, book_author) {
            return Ok(Some(found_id));
        }
    }
    Ok(None)
}

fn matches(str1: &str, str2: Option<&str>) -> bool {
    let str1 = clean_string(str1);
    match str2 {
        Some(s) => str1.contains(&clean_string(s)),
        None => true,
    }
}

fn clean_string(input: &str) -> String {
    let result: String = input.chars().filter(|c| c.is_alphanumeric()).collect();
    result.to_lowercase()
}

fn extract_goodreads_id(url: &str) -> Result<String, ScraperError> {
    let id = url
        .splitn(4, '/')
        .nth(3)
        .expect("Failed to extract Goodreads ID")
        .split('?')
        .next()
        .expect("Failed to extract Goodreads ID")
        .chars()
        .take_while(|c| c.is_numeric())
        .collect::<String>();

    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_book_with_author_1() {
        let book_title = "Babel";
        let book_author = "R. F. Kuang";
        assert_eq!(
            search_book(book_title, Some(book_author)).unwrap(),
            Some("57945316".to_string())
        );
    }

    #[test]
    fn test_search_book_with_author_2() {
        let book_title = "Fire";
        let book_author = "Kristin Cashore";
        assert_eq!(
            search_book(book_title, Some(book_author)).unwrap(),
            Some("6137154".to_string())
        );
    }

    #[test]
    fn test_search_book_without_author_1() {
        let book_title = "Babel";
        assert_eq!(
            search_book(book_title, None).unwrap(),
            Some("57945316".to_string())
        );
    }

    #[test]
    fn test_search_book_without_author_2() {
        let book_title = "Fire";
        assert_eq!(
            search_book(book_title, None).unwrap(),
            Some("6148028".to_string())
        );
    }
}
