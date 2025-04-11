use grscraper::{BookContributor, BookMetadata, BookSeries, MetadataRequestBuilder};

#[tokio::test]
async fn fetch_metadata_by_title_test() {
    let title = "The Last Magician";
    let metadata = MetadataRequestBuilder::default()
        .with_title(title)
        .execute()
        .await
        .unwrap();

    verify_metadata(metadata);
}

#[tokio::test]
async fn fetch_metadata_by_id_test() {
    let id = "30312855";
    let metadata = MetadataRequestBuilder::default()
        .with_id(id)
        .execute()
        .await
        .unwrap();

    verify_metadata(metadata);
}

#[tokio::test]
async fn fetch_metadata_by_isbn_test() {
    let isbn = "1481432079";
    let metadata = MetadataRequestBuilder::default()
        .with_isbn(isbn)
        .execute()
        .await
        .unwrap();

    verify_metadata(metadata);
}

#[tokio::test]
async fn fetch_metadata_by_isbn_bad_time_test() {
    let isbn = " 9788467271300";
    let metadata = MetadataRequestBuilder::default()
        .with_isbn(isbn)
        .execute()
        .await
        .unwrap();

    verify_metadata(metadata);
}

#[tokio::test]
async fn fetch_metadata_by_title_with_author_test() {
    let title = "The Last Magician";
    let author = "Lisa Maxwell";
    let metadata = MetadataRequestBuilder::default()
        .with_title(title)
        .with_author(author)
        .execute()
        .await
        .unwrap();

    verify_metadata(metadata);
}

fn verify_metadata(metadata: Option<BookMetadata>) {
    let expected_series = BookSeries::new("The Last Magician".to_string(), 1.0);
    let expected_contributors = vec![BookContributor::new(
        "Lisa Maxwell".to_string(),
        "Author".to_string(),
    )];
    let expected_genres = vec![
        "Fantasy".to_string(),
        "Young Adult".to_string(),
        "Historical Fiction".to_string(),
        "Time Travel".to_string(),
        "Magic".to_string(),
        "Young Adult Fantasy".to_string(),
        "Historical".to_string(),
        "Fiction".to_string(),
        "Urban Fantasy".to_string(),
        "Audiobook".to_string(),
    ];
    let expected_metadata = BookMetadata::new(
        "The Last Magician".to_string(),
        None,
        Some("<i>Stop the Magician. Steal the book. Save the future.</i><br /><br />In modern-day New York, magic is all but extinct. \
        The remaining few who have an affinity for magic—the Mageus—live in the shadows, hiding who they are. Any Mageus who enters \
        Manhattan becomes trapped by the Brink, a dark energy barrier that confines them to the island. Crossing it means losing their \
        power—and often their lives.<br /><br />Esta is a talented thief, and she’s been raised to steal magical artifacts from the sinister \
        Order that created the Brink. With her innate ability to manipulate time, Esta can pilfer from the past, collecting these artifacts \
        before the Order even realizes she’s there. And all of Esta’s training has been for one final job: traveling back to 1902 to steal an \
        ancient book containing the secrets of the Order—and the Brink—before the Magician can destroy it and doom the Mageus to a hopeless \
        future.<br /><br />But Old New York is a dangerous world ruled by ruthless gangs and secret societies, a world where the very air \
        crackles with magic. Nothing is as it seems, including the Magician himself. And for Esta to save her future, she may have to betray \
        everyone in the past.".to_string()),
        Some("Margaret K. McElderry Books".to_string()),
        Some("2017-07-18T07:00:00Z".parse().unwrap()),
        Some("1481432079".to_string()),
        expected_contributors,
        expected_genres,
        Some(expected_series),
        Some(500),
        Some("English".to_string()),
        Some("https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1468598919i/30312855.jpg".to_string())
    );

    assert_eq!(metadata, Some(expected_metadata));
}
