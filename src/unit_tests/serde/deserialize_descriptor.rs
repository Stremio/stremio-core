use crate::types::addon::{Descriptor, DescriptorFlags, Manifest};
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
