use scraper::error::SelectorErrorKind;

/// Custom error type for handling errors in the Goodreads metadata scraper.
#[derive(Debug)]
pub enum ScraperError {
    /// Error that occurs during the HTTP request to Goodreads, originating from `reqwest`.
    FetchError(reqwest::Error),
    /// Error encountered while parsing the HTML document, originating from `scraper`.
    ParseError(String),
    /// Non-recoverable error encountered while scraping the HTML document. Indicates expected content was missing.
    ScrapeError(String),
    /// Error encountered during JSON serialization, originating from `serde_json`.
    SerializeError(serde_json::Error),
}

impl From<reqwest::Error> for ScraperError {
    fn from(error: reqwest::Error) -> Self {
        ScraperError::FetchError(error)
    }
}

impl From<SelectorErrorKind<'static>> for ScraperError {
    fn from(error: SelectorErrorKind<'static>) -> Self {
        ScraperError::ParseError(error.to_string())
    }
}

impl From<serde_json::Error> for ScraperError {
    fn from(error: serde_json::Error) -> Self {
        ScraperError::SerializeError(error)
    }
}
