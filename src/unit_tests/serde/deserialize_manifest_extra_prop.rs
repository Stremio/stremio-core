use crate::types::addon::{ManifestExtraProp, OptionsLimit};

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
        "ManifestExtraProp deserialized successfully"
    );
}
