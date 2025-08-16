//! # Goodreads Metadata Scraper
//!
//! This library provides a convenient way to fetch and scrape book metadata from Goodreads by providing either
//! an ISBN, a Goodreads book ID, or a combination of title and author. It is designed to be used in applications
//! where access to Goodreads book data is needed, but no official Goodreads API access is available.
//!
//! ## Features
//!
//! - Retrieve metadata by ISBN
//! - Retrieve metadata by Goodreads ID
//! - Retrieve metadata by title (optionally with author for more accurate results)
//! - Structured, typed metadata output for easy access to common book fields (title, author, publication year, etc.)
//! - Query builder pattern for customizable requests
//!
//! ## Usage
//!
//! Add this crate to your dependencies in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! goodreads-metadata-scraper = "0.2.3"
//! ```
//!
//! ## Examples
//!
//! To use this library, create a `MetadataRequestBuilder` to specify the search criteria. Then, use
//! the `execute` method to retrieve the metadata.
//!
//! ### Fetching Metadata by ISBN
//!
//! ```rust
//! use grscraper::MetadataRequestBuilder;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), grscraper::ScraperError> {
//! let isbn = "9780141381473";
//! let metadata = MetadataRequestBuilder::default()
//!     .with_isbn(isbn)
//!     .execute()
//!     .await?
//!     .expect("Book not found");
//!
//! assert_eq!(metadata.title, "The Lightning Thief");
//! println!("{:#?}", metadata);
//! # Ok::<(), grscraper::ScraperError>(())
//! # }
//! ```
//!
//! ### Fetching Metadata by Goodreads ID
//!
//! ```rust
//! use grscraper::MetadataRequestBuilder;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), grscraper::ScraperError> {
//! let goodreads_id = "175254";
//! let metadata = MetadataRequestBuilder::default()
//!     .with_id(goodreads_id)
//!     .execute()
//!     .await?
//!     .expect("Book not found");
//!
//! assert_eq!(metadata.title, "Pride and Prejudice");
//! println!("{:#?}", metadata);
//! # Ok::<(), grscraper::ScraperError>(())
//! # }
//! ```
//!
//! ### Fetching Metadata by Title and Author
//!
//! Providing an author along with the title helps improve the accuracy of the search:
//!
//! ```rust
//! use grscraper::MetadataRequestBuilder;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), grscraper::ScraperError> {
//! let title = "The Last Magician";
//! let author = "Lisa Maxwell";
//! let metadata = MetadataRequestBuilder::default()
//!     .with_title(title)
//!     .with_author(author)
//!     .execute()
//!     .await?
//!     .expect("Book not found");
//!
//! assert_eq!(metadata.title, title);
//! println!("{:#?}", metadata);
//! # Ok::<(), grscraper::ScraperError>(())
//! # }
//! ```
//!
//! ## Limitations
//!
//! - Since this library relies on web scraping, it may be sensitive to changes in Goodreads' website structure.
//! - This library is intended for personal or small-scale use, as frequent requests to Goodreads may be rate-limited.
//!
//! **Note:** When running tests, it is highly recommended to run them with the `--test-threads=1` flag to avoid rate-limiting issues with Goodreads.
//!
//! ## License
//!
//! This project is licensed under the GNU General Public License (GPL).
//!

mod errors;
mod goodreads_id_fetcher;
mod metadata_fetcher;
mod request_builder;

pub use errors::ScraperError;
pub use metadata_fetcher::BookContributor;
pub use metadata_fetcher::BookMetadata;
pub use metadata_fetcher::BookSeries;
pub use request_builder::MetadataRequestBuilder;
