# Goodreads Metadata Scraper

[![Crates.io](https://img.shields.io/crates/v/goodreads_metadata_scraper.svg)](https://crates.io/crates/goodreads_metadata_scraper)
[![Documentation](https://docs.rs/goodreads-metadata-scraper/badge.svg)](https://docs.rs/goodreads_metadata_scraper)

An async Rust library to fetch and scrape book metadata from Goodreads by using an ISBN, Goodreads book ID, or a combination of title and author. This library is useful for applications that need book data from Goodreads without access to an official API.

## Features

- Retrieve metadata by ISBN
- Retrieve metadata by Goodreads ID
- Retrieve metadata by title (optionally with author for better accuracy)
- Structured metadata output with fields such as title, author, publication year, and more
- Query builder pattern for flexible request customization

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
goodreads-metadata-scraper = "0.2.4"
```

## Usage

Here are a few examples of how to use the library. The primary entry point is `MetadataRequestBuilder`, which allows you to specify search criteria before calling `execute` to fetch metadata.

### Fetching Metadata by ISBN

```rust
use grscraper::MetadataRequestBuilder;

let isbn = "9780141381473";
let metadata = MetadataRequestBuilder::default()
    .with_isbn(isbn)
    .execute()
    .await?
    .expect("Book not found");

assert_eq!(metadata.title, "The Lightning Thief");
println!("{:#?}", metadata);
```

### Fetching Metadata by Goodreads ID

```rust
use grscraper::MetadataRequestBuilder;

let goodreads_id = "175254";
let metadata = MetadataRequestBuilder::default()
    .with_id(goodreads_id)
    .execute()
    .await?
    .expect("Book not found");

assert_eq!(metadata.title, "Pride and Prejudice");
println!("{:#?}", metadata);
```

### Fetching Metadata by Title and Author

For better accuracy, you can also specify an author along with the title:

```rust
use grscraper::MetadataRequestBuilder;

let title = "The Last Magician";
let author = "Lisa Maxwell";
let metadata = MetadataRequestBuilder::default()
    .with_title(title)
    .with_author(author)
    .execute()
    .await?
    .expect("Book not found");

assert_eq!(metadata.title, title);
println!("{:#?}", metadata);
```

## Metadata Structure

The returned metadata is structured as follows:

```rust
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
```

## Error Handling

This crate uses a custom error type, `ScraperError`, which handles errors that may occur during the metadata fetching and parsing process. `ScraperError` includes:

- `FetchError`: Errors during HTTP requests (from `reqwest`)
- `ParseError`: HTML parsing errors (from `scraper`)
- `SerializeError`: JSON serialization errors (from `serde_json`)
- `ScrapeError`: Non-recoverable error encountered while scraping the HTML document. Indicates expected content was missing.

## Limitations

- As this library relies on web scraping, any changes in Goodreads' HTML structure may break functionality.
- This library is intended for personal or small-scale use, as frequent requests to Goodreads may be rate-limited.

**Note:** When running tests, it is highly recommended to run them with the `--test-threads=1` flag to avoid rate-limiting issues with Goodreads.
