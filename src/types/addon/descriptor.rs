use crate::types::addon::{Manifest, ManifestPreview};
use serde::{Deserialize, Serialize};
use url::Url;

/// Addon descriptor
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub manifest: Manifest,
    pub transport_url: Url,
    #[serde(default)]
    pub flags: DescriptorFlags,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorPreview {
    pub manifest: ManifestPreview,
    pub transport_url: Url,
}

impl From<&Descriptor> for DescriptorPreview {
    fn from(value: &Descriptor) -> Self {
        Self {
            manifest: (&value.manifest).into(),
            transport_url: value.transport_url.clone(),
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorFlags {
    #[serde(default)]
    pub official: bool,
    #[serde(default)]
    pub protected: bool,
}
