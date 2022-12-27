use crate::types::addon::DescriptorPreview;
use crate::types::resource::{MetaItem, MetaItemPreview, Stream, Subtitles};
use derive_more::TryInto;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Clone, TryInto, Serialize, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum ResourceResponse {
    Metas {
        metas: Vec<MetaItemPreview>,
    },
    #[serde(rename_all = "camelCase")]
    MetasDetailed {
        metas_detailed: Vec<MetaItem>,
    },
    Meta {
        meta: MetaItem,
    },
    Streams {
        streams: Vec<Stream>,
    },
    Subtitles {
        subtitles: Vec<Subtitles>,
    },
    Addons {
        addons: Vec<DescriptorPreview>,
    },
}

impl<'de> Deserialize<'de> for ResourceResponse {
    fn deserialize<D>(deserializer: D) -> Result<ResourceResponse, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let mut value = match value {
            serde_json::Value::Object(value) => value,
            _ => {
                return Err(serde::de::Error::custom(
                    "Cannot deserialize as ResourceResponse",
                ))
            }
        };
        if let Some(value) = value.get_mut("metas") {
            Ok(ResourceResponse::Metas {
                metas: serde_json::from_value(value.take()).map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("metasDetailed") {
            Ok(ResourceResponse::MetasDetailed {
                metas_detailed: serde_json::from_value(value.take())
                    .map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("meta") {
            Ok(ResourceResponse::Meta {
                meta: serde_json::from_value(value.take()).map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("streams") {
            Ok(ResourceResponse::Streams {
                streams: serde_json::from_value(value.take()).map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("subtitles") {
            Ok(ResourceResponse::Subtitles {
                subtitles: serde_json::from_value(value.take())
                    .map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("addons") {
            Ok(ResourceResponse::Addons {
                addons: serde_json::from_value(value.take()).map_err(serde::de::Error::custom)?,
            })
        } else {
            Err(serde::de::Error::custom(
                "Cannot deserialize as ResourceResponse",
            ))
        }
    }
}
