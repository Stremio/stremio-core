use crate::types::addon::ManifestExtra;

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
        "ManifestExtra deserialized successfully"
    );
}
