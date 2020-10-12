use crate::types::addon::{ManifestCatalog, ManifestExtra};

#[test]
fn deserialize_manifest_catalog() {
    let manifest_catalogs = vec![
        ManifestCatalog {
            type_: "type".to_owned(),
            id: "id".to_owned(),
            name: Some("name".to_owned()),
            extra: ManifestExtra::Full { props: vec![] },
        },
        ManifestCatalog {
            type_: "type".to_owned(),
            id: "id".to_owned(),
            name: None,
            extra: ManifestExtra::Full { props: vec![] },
        },
    ];
    let manifest_catalogs_json = r#"
    [
        {
            "type": "type",
            "id": "id",
            "name": "name",
            "extra": []
        },
        {
            "type": "type",
            "id": "id",
            "name": null,
            "extra": []
        }
    ]
    "#;
    let manifest_catalogs_deserialize: Vec<ManifestCatalog> =
        serde_json::from_str(&manifest_catalogs_json).unwrap();
    assert_eq!(
        manifest_catalogs, manifest_catalogs_deserialize,
        "ManifestCatalog deserialized successfully"
    );
}
