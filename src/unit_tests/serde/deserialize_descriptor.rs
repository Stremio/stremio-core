use crate::types::addon::{
    Descriptor, DescriptorFlags, Manifest, ManifestCatalog, ManifestExtra, ManifestExtraProp,
    OptionsLimit,
};
use url::Url;

#[test]
fn deserialize_descriptor() {
    let descriptors = vec![
        Descriptor {
            manifest: Manifest::default(),
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: DescriptorFlags::default(),
        },
        Descriptor {
            manifest: Manifest::default(),
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: DescriptorFlags::default(),
        },
    ];
    let descriptors_json = format!(
        r#"
        [
            {{
                "manifest": {},
                "transportUrl": "https://transport_url",
                "flags": {}
            }},
            {{
                "manifest": {},
                "transportUrl": "https://transport_url"
            }}
        ]
        "#,
        serde_json::to_string(&Manifest::default()).unwrap(),
        serde_json::to_string(&DescriptorFlags::default()).unwrap(),
        serde_json::to_string(&Manifest::default()).unwrap(),
    );
    let descriptors_deserialize: Vec<Descriptor> = serde_json::from_str(&descriptors_json).unwrap();
    assert_eq!(
        descriptors, descriptors_deserialize,
        "Descriptor deserialized successfully"
    );
}

#[test]
fn deserialize_manifest_catalog() {
    let manifest_catalogs = vec![
        ManifestCatalog {
            type_name: "type_name".to_owned(),
            id: "id".to_owned(),
            name: Some("name".to_owned()),
            extra: ManifestExtra::Full { props: vec![] },
        },
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
        ManifestExtra::Short {
            required: vec!["required".to_owned()],
            supported: vec!["supported".to_owned()],
        },
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
        ManifestExtraProp {
            name: "name".to_owned(),
            is_required: true,
            options: Some(vec!["option".to_owned()]),
            options_limit: OptionsLimit(2),
        },
        ManifestExtraProp {
            name: "name".to_owned(),
            is_required: false,
            options: None,
            options_limit: OptionsLimit(1),
        },
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
        DescriptorFlags {
            official: true,
            protected: true,
        },
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
