use derive_more::TryInto;
use serde::{de::Deserializer, Deserialize, Serialize};
use serde_with::{serde_as, VecSkipError};

use crate::{
    addon_transport::legacy::SubtitlesResult,
    types::{
        addon::DescriptorPreview,
        resource::{MetaItem, MetaItemPreview, Stream, Subtitles},
        watch_status,
    },
};

/// Resource Response from an addon.
///
/// Deserializing the struct from json will skip any invalid Vec items
/// and will skip any unknown to the variants fields.
///
/// # Examples
/// ```
/// use stremio_core::types::addon::ResourceResponse;
///
/// // watchStatus response
/// {
///     let success_watch_status = ResourceResponse::WatchStatus {
///         watch_status: true,
///     };
///
///     let success_json = serde_json::json!({
///         "watchStatus": true
///     });
///
///     assert_eq!(serde_json::from_value::<ResourceResponse>(success_json).expect("Should deserialize"), success_watch_status);
///
///     let error_watch_status = ResourceResponse::WatchStatus {
///         watch_status: false,
///     };
///     let error_json = serde_json::json!({
///         "watchStatus": false
///     });
///
///     assert_eq!(serde_json::from_value::<ResourceResponse>(error_json).expect("Should deserialize"), error_watch_status);
///
///     let bad_json = serde_json::json!({
///         "watchStatus": "bad value"
///     });
///
///     // we default to `false` if the value in watchStatus is not boolean.
///     assert_eq!(serde_json::from_value::<ResourceResponse>(bad_json).expect("Should deserialize"), error_watch_status);
/// }
/// ```
#[derive(Clone, TryInto, Serialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
#[serde_as]
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
    #[serde(rename_all = "camelCase")]
    WatchStatus {
        watch_status: bool,
    },
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct SkipError<T: for<'a> Deserialize<'a> + Serialize>(
    #[serde_as(as = "VecSkipError<_>")] Vec<T>,
);

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
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::Metas { metas: skip.0 })
        } else if let Some(value) = value.get_mut("metasDetailed") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::MetasDetailed {
                metas_detailed: skip.0,
            })
        } else if let Some(value) = value.get_mut("meta") {
            Ok(ResourceResponse::Meta {
                meta: serde_json::from_value(value.take()).map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("streams") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::Streams { streams: skip.0 })
        } else if let Some(value) = value.get_mut("subtitles") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::Subtitles { subtitles: skip.0 })
        } else if let Some(value) = value.get_mut("addons") {
            let skip = serde_json::from_value::<SkipError<_>>(value.take())
                .map_err(serde::de::Error::custom)?;

            Ok(ResourceResponse::Addons { addons: skip.0 })
        } else if let Some(value) = value.get_mut("watchStatus") {
            let success = serde_json::from_value::<bool>(value.take()).unwrap_or_default();

            Ok(ResourceResponse::WatchStatus {
                watch_status: success,
            })
        } else {
            Err(serde::de::Error::custom(
                "Cannot deserialize as ResourceResponse",
            ))
        }
    }
}

impl From<Vec<MetaItemPreview>> for ResourceResponse {
    fn from(metas: Vec<MetaItemPreview>) -> Self {
        ResourceResponse::Metas { metas }
    }
}
impl From<MetaItem> for ResourceResponse {
    fn from(meta: MetaItem) -> Self {
        ResourceResponse::Meta { meta }
    }
}
impl From<Vec<Stream>> for ResourceResponse {
    fn from(streams: Vec<Stream>) -> Self {
        ResourceResponse::Streams { streams }
    }
}

impl From<watch_status::Response> for ResourceResponse {
    fn from(response: watch_status::Response) -> Self {
        ResourceResponse::WatchStatus {
            watch_status: response.success,
        }
    }
}

impl TryFrom<ResourceResponse> for watch_status::Response {
    type Error = &'static str;

    fn try_from(resource_response: ResourceResponse) -> Result<Self, Self::Error> {
        match resource_response {
            ResourceResponse::WatchStatus { watch_status } => Ok(Self {
                success: watch_status,
            }),
            _ => Err("Bad WatchStatus response"),
        }
    }
}

/// Legacy SubtitlesResult
impl From<SubtitlesResult> for ResourceResponse {
    fn from(subtitles_result: SubtitlesResult) -> Self {
        ResourceResponse::Subtitles {
            subtitles: subtitles_result.all,
        }
    }
}
