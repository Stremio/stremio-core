use crate::types::addon::{Manifest, ManifestPreview};
use serde::{Deserialize, Serialize};
use url::Url;

/// Addon descriptor
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub manifest: Manifest,
    pub transport_url: Url,
    #[serde(default)]
    pub flags: DescriptorFlags,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct DescriptorPreview {
    pub manifest: ManifestPreview,
    pub transport_url: Url,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct DescriptorFlags {
    #[serde(default)]
    pub official: bool,
    #[serde(default)]
    pub protected: bool,
}
