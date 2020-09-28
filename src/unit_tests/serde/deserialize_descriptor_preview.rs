use crate::types::addon::{DescriptorPreview, ManifestPreview};
use semver::Version;
use url::Url;

#[test]
fn deserialize_descriptor_preview() {
    let descriptor_previews = vec![
        DescriptorPreview {
            manifest: ManifestPreview {
                id: "id1".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                description: Some("description".to_owned()),
                logo: Some("logo".to_owned()),
                background: Some("background".to_owned()),
                types: vec!["type".to_owned()],
            },
            transport_url: Url::parse("https://transport_url").unwrap(),
        },
        DescriptorPreview {
            manifest: ManifestPreview {
                id: "id2".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                description: None,
                logo: None,
                background: None,
                types: vec![],
            },
            transport_url: Url::parse("https://transport_url").unwrap(),
        },
    ];
    let descriptor_previews_json = r#"
    [
        {
            "manifest": {
                "id": "id1",
                "version": "0.0.1",
                "name": "name",
                "description": "description",
                "logo": "logo",
                "background": "background",
                "types": [
                    "type"
                ]
            },
            "transportUrl": "https://transport_url"
        },
        {
            "manifest": {
                "id": "id2",
                "version": "0.0.1",
                "name": "name",
                "description": null,
                "logo": null,
                "background": null,
                "types": []
            },
            "transportUrl": "https://transport_url"
        }
    ]
    "#;
    let descriptor_previews_deserialize: Vec<DescriptorPreview> =
        serde_json::from_str(&descriptor_previews_json).unwrap();
    assert_eq!(
        descriptor_previews, descriptor_previews_deserialize,
        "DescriptorPreview deserialized successfully"
    );
}
