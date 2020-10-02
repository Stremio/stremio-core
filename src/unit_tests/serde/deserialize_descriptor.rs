use crate::types::addon::{Descriptor, DescriptorFlags, Manifest, ManifestExtraProp, OptionsLimit};
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
