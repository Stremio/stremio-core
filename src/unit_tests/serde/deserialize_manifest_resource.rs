use crate::types::addon::ManifestResource;

#[test]
fn deserialize_manifest_resource() {
    let manifest_resources = vec![
        ManifestResource::Short("resource".to_owned()),
        ManifestResource::Full {
            name: "name".to_owned(),
            types: Some(vec!["type".to_owned()]),
            id_prefixes: Some(vec!["id_prefix".to_owned()]),
        },
        ManifestResource::Full {
            name: "name".to_owned(),
            types: None,
            id_prefixes: None,
        },
    ];
    let manifest_resources_json = r#"
    [
        "resource",
        {
            "name": "name",
            "types": [
                "type"
            ],
            "idPrefixes": [
                "id_prefix"
            ]
        },
        {
            "name": "name",
            "types": null,
            "idPrefixes": null
        }
    ]
    "#;
    let manifest_resources_deserialize: Vec<ManifestResource> =
        serde_json::from_str(&manifest_resources_json).unwrap();
    assert_eq!(
        manifest_resources, manifest_resources_deserialize,
        "ManifestResource deserialized successfully"
    );
}
