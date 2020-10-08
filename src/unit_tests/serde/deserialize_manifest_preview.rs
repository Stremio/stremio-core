use crate::types::addon::ManifestPreview;
use semver::Version;

#[test]
fn deserialize_manifest_preview() {
    let manifest_previews = vec![
        ManifestPreview {
            id: "id1".to_owned(),
            version: Version::new(0, 0, 1),
            name: "name".to_owned(),
            description: Some("description".to_owned()),
            logo: Some("logo".to_owned()),
            background: Some("background".to_owned()),
            types: vec!["type".to_owned()],
        },
        ManifestPreview {
            id: "id2".to_owned(),
            version: Version::new(0, 0, 1),
            name: "name".to_owned(),
            description: None,
            logo: None,
            background: None,
            types: vec![],
        },
    ];
    let manifest_previews_json = r#"
    [
        {
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
        {
            "id": "id2",
            "version": "0.0.1",
            "name": "name",
            "description": null,
            "logo": null,
            "background": null,
            "types": []
        }
    ]
    "#;
    let manifest_previews_deserialize: Vec<ManifestPreview> =
        serde_json::from_str(&manifest_previews_json).unwrap();
    assert_eq!(
        manifest_previews, manifest_previews_deserialize,
        "ManifestPreview deserialized successfully"
    );
}
