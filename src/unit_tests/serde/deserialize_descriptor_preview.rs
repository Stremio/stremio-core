use crate::types::addon::{DescriptorPreview, ManifestPreview};
use url::Url;

#[test]
fn deserialize_descriptor_preview() {
    let descriptor_preview = DescriptorPreview {
        manifest: ManifestPreview::default(),
        transport_url: Url::parse("https://transport_url").unwrap(),
    };
    let descriptor_preview_json = format!(
        r#"
        {{
            "manifest": {},
            "transportUrl": "https://transport_url"
        }}
        "#,
        serde_json::to_string(&ManifestPreview::default()).unwrap(),
    );
    let descriptor_preview_deserialize = serde_json::from_str(&descriptor_preview_json).unwrap();
    assert_eq!(
        descriptor_preview, descriptor_preview_deserialize,
        "DescriptorPreview deserialized successfully"
    );
}
