use chrono::{DateTime, Utc};
use derive_new::new;
use scraper::error::SelectorErrorKind;

#[derive(Debug)]
pub enum ScraperError {
    FetchError(reqwest::Error),
    ParseError(scraper::error::SelectorErrorKind<'static>),
    SerializeError(serde_json::Error),
}

impl From<reqwest::Error> for ScraperError {
    fn from(error: reqwest::Error) -> Self {
        ScraperError::FetchError(error)
    }
}

impl From<SelectorErrorKind<'static>> for ScraperError {
    fn from(error: SelectorErrorKind<'static>) -> Self {
        ScraperError::ParseError(error)
    }
}

impl From<serde_json::Error> for ScraperError {
    fn from(error: serde_json::Error) -> Self {
        ScraperError::SerializeError(error)
    }
}

#[derive(Debug, new, PartialEq)]
pub struct BookMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub publication_date: Option<DateTime<Utc>>,
    pub isbn: Option<String>,
    pub contributors: Vec<Contributor>,
    pub genres: Vec<String>,
    pub series: Option<BookSeries>,
    pub image_url: Option<String>,
}

#[derive(Debug, new, PartialEq)]
pub struct Contributor {
    pub name: String,
    pub role: String,
}

#[derive(Debug, new, PartialEq)]
pub struct BookSeries {
    pub title: String,
    pub number: f32,
}
