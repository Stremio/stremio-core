use crate::types::resource::Subtitles;
use url::Url;

#[test]
fn deserialize_subtitles() {
    let subtitles = vec![Subtitles {
        id: "id".to_owned(),
        lang: "lang".to_owned(),
        url: Url::parse("https://url").unwrap(),
    }];
    let subtitles_json = r#"
    [
        {
            "id": "id",
            "lang": "lang",
            "url": "https://url"
        }
    ]
    "#;
    let subtitles_deserialize: Vec<Subtitles> = serde_json::from_str(&subtitles_json).unwrap();
    assert_eq!(
        subtitles, subtitles_deserialize,
        "Subtitles deserialized successfully"
    );
}
