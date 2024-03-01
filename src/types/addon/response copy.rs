use derive_more::TryInto;
use serde::{de::Deserializer, Deserialize, Serialize};
use serde_with::{serde_as, VecSkipError};

use crate::types::{
    addon::DescriptorPreview,
    resource::{MetaItem, MetaItemPreview, Stream, Subtitles},
};

/// Resource response that handles the optional cache values defined in the SDK.
/// This is a shim struct to avoid the custom Deserialize impl that skips those fields
/// for the [`ResourceResponse`] enum.
///
/// See <https://github.com/Stremio/stremio-addon-sdk/tree/master/docs/api/requests>
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceResponseCache {
    /// (in seconds) which sets the `Cache-Control` header to `max-age=$cacheMaxAge` and overwrites the global cache time set in serveHTTP options
    pub cache_max_age: Option<u64>,
    /// (in seconds) which sets the `Cache-Control` header to `stale-while-revalidate=$staleRevalidate`
    pub stale_revalidate: Option<u64>,
    /// (in seconds) which sets the `Cache-Control` header to `stale-if-error=$staleError`
    pub stale_error: Option<u64>,
    #[serde(flatten)]
    pub resource: ResourceResponse,
}

/// Resource Response from an addon.
///
/// Deserializing the struct from json will skip any invalid Vec items
/// and will skip any unknown to the variants fields.
/// This requires
/// ```
/// use stremio_core::types::addon::ResourceResponse;
///
/// {
/// }
///
/// ```
#[derive(Clone, TryInto, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
#[serde_as]
pub enum ResourceResponse {
    Metas {
        #[serde_as(as = "VecSkipError<_>")]
        metas: Vec<MetaItemPreview>,
    },
    #[serde(rename_all = "camelCase")]
    MetasDetailed {
        #[serde_as(as = "VecSkipError<_>")]
        metas_detailed: Vec<MetaItem>,
    },
    Meta {
        meta: MetaItem,
    },
    Streams {
        #[serde_as(as = "VecSkipError<_>")]
        streams: Vec<Stream>,
    },
    Subtitles {
        #[serde_as(as = "VecSkipError<_>")]
        subtitles: Vec<Subtitles>,
    },
    Addons {
        #[serde_as(as = "VecSkipError<_>")]
        addons: Vec<DescriptorPreview>,
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

        tracing::info!("Response: {value:#?}");
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
        } else {
            Err(serde::de::Error::custom(
                "Cannot deserialize as ResourceResponse",
            ))
        }
    }
}
