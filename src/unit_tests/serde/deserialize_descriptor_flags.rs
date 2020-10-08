use crate::types::addon::DescriptorFlags;

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
        "DescriptorFlags deserialized successfully"
    );
}
