use crate::types::addon::{
    Descriptor, DescriptorFlags, Manifest, ManifestCatalog, ManifestExtra, ManifestExtraProp,
    ManifestResource, OptionsLimit,
};
use semver::Version;
use url::Url;

#[test]
fn deserialize_descriptor() {
    let descriptors = vec![
        Descriptor {
            manifest: Manifest {
                id: "id".to_owned(),
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
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: DescriptorFlags {
                official: false,
                protected: false,
            },
        },
        Descriptor {
            manifest: Manifest {
                id: "id".to_owned(),
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
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: DescriptorFlags {
                official: false,
                protected: false,
            },
        },
    ];
    let descriptors_json = r#"
    [
        {
            "manifest": {
                "id": "id",
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
            "transportUrl": "https://transport_url/",
            "flags": {
                "official": false,
                "protected": false
            }
        },
        {
            "manifest": {
                "id": "id",
                "version": "0.0.1",
                "name": "name",
                "types": [],
                "resources": [],
                "idPrefixes": null
            },
            "transportUrl": "https://transport_url/"
        }
    ]
    "#;
    let descriptors_deserialize: Vec<Descriptor> = serde_json::from_str(&descriptors_json).unwrap();
    assert_eq!(
        descriptors, descriptors_deserialize,
        "descriptor deserialized successfully"
    );
}

#[test]
fn deserialize_manifest_resource() {
    let manifest_resources = vec![
        ManifestResource::Short("resource".to_owned()),
        // ALL fields are defined with SOME value
        ManifestResource::Full {
            name: "name".to_owned(),
            types: Some(vec!["type".to_owned()]),
            id_prefixes: Some(vec!["id_prefix".to_owned()]),
        },
        // ALL NONEs are set to null.
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
        "manifest resource deserialized successfully"
    );
}

#[test]
fn deserialize_manifest_catalog() {
    let manifest_catalogs = vec![
        // ALL fields are defined with SOME value
        ManifestCatalog {
            type_name: "type_name".to_owned(),
            id: "id".to_owned(),
            name: Some("name".to_owned()),
            extra: ManifestExtra::Full { props: vec![] },
        },
        // ALL NONEs are set to null.
        ManifestCatalog {
            type_name: "type_name".to_owned(),
            id: "id".to_owned(),
            name: None,
            extra: ManifestExtra::Full { props: vec![] },
        },
    ];
    let manifest_catalogs_json = r#"
    [
        {
            "type": "type_name",
            "id": "id",
            "name": "name",
            "extra": []
        },
        {
            "type": "type_name",
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
        "manifest catalog deserialized successfully"
    );
}

#[test]
fn deserialize_manifest_extra() {
    let manifest_extra = vec![
        ManifestExtra::Full { props: vec![] },
        // ALL fields are defined
        ManifestExtra::Short {
            required: vec!["required".to_owned()],
            supported: vec!["supported".to_owned()],
        },
        // serde(default) are omited
        ManifestExtra::Short {
            required: vec![],
            supported: vec![],
        },
    ];
    let manifest_extra_json = r#"
    [
        {
            "extra": []
        },
        {
            "extraRequired": [
                "required"
            ],
            "extraSupported": [
                "supported"
            ]
        },
        {}
    ]
    "#;
    let manifest_extra_deserialize: Vec<ManifestExtra> =
        serde_json::from_str(&manifest_extra_json).unwrap();
    assert_eq!(
        manifest_extra, manifest_extra_deserialize,
        "manifest extra deserialized successfully"
    );
}

#[test]
fn deserialize_manifest_extra_prop() {
    let manifest_extra_props = vec![
        // ALL fields are defined with SOME value
        ManifestExtraProp {
            name: "name".to_owned(),
            is_required: true,
            options: Some(vec!["option".to_owned()]),
            options_limit: OptionsLimit(2),
        },
        // serde(default) are omited
        ManifestExtraProp {
            name: "name".to_owned(),
            is_required: false,
            options: None,
            options_limit: OptionsLimit(1),
        },
        // ALL NONEs are set to null.
        ManifestExtraProp {
            name: "name".to_owned(),
            is_required: false,
            options: None,
            options_limit: OptionsLimit(0),
        },
    ];
    let manifest_extra_props_json = r#"
    [
        {
            "name": "name",
            "isRequired": true,
            "options": [
                "option"
            ],
            "optionsLimit": 2
        },
        {
            "name": "name",
            "options": null
        },
        {
            "name": "name",
            "isRequired": false,
            "options": null,
            "optionsLimit": 0
        }
    ]
    "#;
    let manifest_extra_props_deserialize: Vec<ManifestExtraProp> =
        serde_json::from_str(&manifest_extra_props_json).unwrap();
    assert_eq!(
        manifest_extra_props, manifest_extra_props_deserialize,
        "manifest extra prop deserialized successfully"
    );
}

#[test]
fn deserialize_descriptor_flags() {
    let descriptor_flags = vec![
        // ALL fields are defined
        DescriptorFlags {
            official: true,
            protected: true,
        },
        // serde(default) are omited
        DescriptorFlags {
            official: false,
            protected: false,
        },
    ];
    let descriptor_flags_json = r#"
    [
        {
            "official": true,
            "protected": true
        },
        {}
    ]
    "#;
    let descriptor_flags_deserialize: Vec<DescriptorFlags> =
        serde_json::from_str(&descriptor_flags_json).unwrap();
    assert_eq!(
        descriptor_flags, descriptor_flags_deserialize,
        "descriptor flags deserialized successfully"
    );
}
