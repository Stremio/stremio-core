use crate::types::addon::Manifest;
use semver::Version;

#[test]
fn deserialize_manifest() {
    let manifests = vec![
        Manifest {
            id: "id1".to_owned(),
            version: Version::new(0, 0, 1),
            name: "name".to_owned(),
            contact_email: Some("contact_email".to_owned()),
            description: Some("description".to_owned()),
            logo: Some("logo".to_owned()),
            background: Some("background".to_owned()),
            types: vec!["type".to_owned()],
            resources: vec![],
            id_prefixes: Some(vec!["id_prefix".to_owned()]),
            catalogs: vec![],
            addon_catalogs: vec![],
            behavior_hints: vec![("behavior_hint".to_owned(), serde_json::Value::Bool(true))]
                .iter()
                .cloned()
                .collect(),
        },
        Manifest {
            id: "id2".to_owned(),
            version: Version::new(0, 0, 1),
            name: "name".to_owned(),
            contact_email: None,
            description: None,
            logo: None,
            background: None,
            types: vec![],
            resources: vec![],
            id_prefixes: None,
            catalogs: vec![],
            addon_catalogs: vec![],
            behavior_hints: vec![].iter().cloned().collect(),
        },
    ];
    let manifests_json = r#"
    [
        {
            "id": "id1",
            "version": "0.0.1",
            "name": "name",
            "contactEmail": "contact_email",
            "description": "description",
            "logo": "logo",
            "background": "background",
            "types": [
                "type"
            ],
            "resources": [],
            "idPrefixes": [
                "id_prefix"
            ],
            "catalogs": [],
            "addonCatalogs": [],
            "behaviorHints": {
                "behavior_hint": true
            }
        },
        {
            "id": "id2",
            "version": "0.0.1",
            "name": "name",
            "contactEmail": null,
            "description": null,
            "logo": null,
            "background": null,
            "types": [],
            "resources": [],
            "idPrefixes": null
        }
    ]
    "#;
    let manifests_deserialize: Vec<Manifest> = serde_json::from_str(&manifests_json).unwrap();
    assert_eq!(
        manifests, manifests_deserialize,
        "Manifest deserialized successfully"
    );
}
