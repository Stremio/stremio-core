use crate::types::addon::{Manifest, ManifestPreview};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorFlags {
    #[serde(default)]
    pub official: bool,
    #[serde(default)]
    pub protected: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorPreview {
    pub manifest: ManifestPreview,
    pub transport_url: Url,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub manifest: Manifest,
    pub transport_url: Url,
    #[serde(default)]
    pub flags: DescriptorFlags,
}
