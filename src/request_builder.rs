use crate::{
    errors::ScraperError,
    goodreads_id_fetcher::{
        fetch_id_from_isbn, fetch_id_from_title, fetch_id_from_title_and_author, verify_id_exists,
    },
    metadata_fetcher::{fetch_metadata, BookMetadata},
};

pub trait RequestState {}
pub struct EmptyState;
pub struct IdState(String);
pub struct IsbnState(String);
pub struct TitleState(String);
pub struct TitleWithAuthorState(String, String);

impl RequestState for EmptyState {}
impl RequestState for IdState {}
impl RequestState for IsbnState {}
impl RequestState for TitleState {}
impl RequestState for TitleWithAuthorState {}

pub struct MetadataRequestBuilder<T: RequestState> {
    state: T,
}

impl MetadataRequestBuilder<EmptyState> {
    pub fn new() -> Self {
        MetadataRequestBuilder { state: EmptyState }
    }

    pub fn with_id(self, id: &str) -> MetadataRequestBuilder<IdState> {
        MetadataRequestBuilder {
            state: IdState(id.to_string()),
        }
    }

    pub fn with_isbn(self, isbn: &str) -> MetadataRequestBuilder<IsbnState> {
        MetadataRequestBuilder {
            state: IsbnState(isbn.to_string()),
        }
    }

    pub fn with_title(self, title: &str) -> MetadataRequestBuilder<TitleState> {
        MetadataRequestBuilder {
            state: TitleState(title.to_string()),
        }
    }
}

impl MetadataRequestBuilder<TitleState> {
    pub fn with_author(self, author: &str) -> MetadataRequestBuilder<TitleWithAuthorState> {
        MetadataRequestBuilder {
            state: TitleWithAuthorState(self.state.0, author.to_string()),
        }
    }

    pub fn execute(&self) -> Result<Option<BookMetadata>, ScraperError> {
        let title = &self.state.0;
        let goodreads_id = fetch_id_from_title(title)?;
        match goodreads_id {
            Some(id) => Ok(Some(fetch_metadata(&id)?)),
            None => Ok(None),
        }
    }
}

impl MetadataRequestBuilder<IdState> {
    pub fn execute(&self) -> Result<Option<BookMetadata>, ScraperError> {
        let id = &self.state.0;
        if !verify_id_exists(id) {
            return Ok(None);
        }
        Ok(Some(fetch_metadata(id)?))
    }
}

impl MetadataRequestBuilder<IsbnState> {
    pub fn execute(&self) -> Result<Option<BookMetadata>, ScraperError> {
        let isbn = &self.state.0;
        let goodreads_id = fetch_id_from_isbn(isbn)?;
        match goodreads_id {
            Some(id) => Ok(Some(fetch_metadata(&id)?)),
            None => Ok(None),
        }
    }
}

impl MetadataRequestBuilder<TitleWithAuthorState> {
    pub fn execute(&self) -> Result<Option<BookMetadata>, ScraperError> {
        let title = &self.state.0;
        let author = &self.state.1;
        let goodreads_id = fetch_id_from_title_and_author(title, author)?;
        match goodreads_id {
            Some(id) => Ok(Some(fetch_metadata(&id)?)),
            None => Ok(None),
        }
    }
}
