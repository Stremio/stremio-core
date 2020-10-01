use crate::types::resource::Link;
use url::Url;

#[test]
fn deserialize_link() {
    let link = Link {
        name: "name".to_owned(),
        category: "category".to_owned(),
        url: Url::parse("https://url").unwrap(),
    };
    let link_json = r#"
    {
        "name": "name",
        "category": "category",
        "url": "https://url"
    }
    "#;
    let link_deserialize = serde_json::from_str(&link_json).unwrap();
    assert_eq!(link, link_deserialize, "Link deserialized successfully");
}
