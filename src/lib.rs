mod errors;
mod goodreads_id_fetcher;
mod metadata_fetcher;
mod request_builder;

pub use errors::ScraperError;
pub use metadata_fetcher::BookContributor;
pub use metadata_fetcher::BookMetadata;
pub use metadata_fetcher::BookSeries;
pub use request_builder::MetadataRequestBuilder;
