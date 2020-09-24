use crate::types::resource::StreamSource;
use url::Url;

#[test]
fn deserialize_stream_source_url() {
    let url = StreamSource::Url {
        url: Url::parse("https://url").unwrap(),
    };
    let url_json = r#"
    {
        "url": "https://url/"
    }
    "#;
    let url_deserialize = serde_json::from_str(&url_json).unwrap();
    assert_eq!(
        url, url_deserialize,
        "Url deserialized successfully"
    );
}
