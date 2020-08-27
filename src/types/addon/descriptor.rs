use crate::types::addon::{Manifest, ManifestPreview};
use serde::{Deserialize, Serialize};

pub type TransportUrl = String;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorFlags {
    #[serde(default)]
    pub official: bool,
    #[serde(default)]
    pub protected: bool,
    #[serde(flatten)]
    #[serde(default)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorPreview {
    pub manifest: ManifestPreview,
    pub transport_url: TransportUrl,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub manifest: Manifest,
    pub transport_url: TransportUrl,
    #[serde(default)]
    pub flags: DescriptorFlags,
}
