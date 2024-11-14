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
