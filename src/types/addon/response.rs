use derive_more::TryInto;
use serde::{de::Deserializer, Deserialize, Serialize};
use serde_with::{serde_as, VecSkipError};

use crate::types::{
    addon::Descriptor,
    resource::{MetaItem, MetaItemPreview, Stream, Subtitles},
};

/// Resource response that handles the optional cache values defined in the SDK.
/// This is a shim struct to avoid the custom Deserialize impl that skips those fields
/// for the [`ResourceResponse`] enum.
///
/// See <https://github.com/Stremio/stremio-addon-sdk/tree/master/docs/api/requests>
///
/// - [`ResourceResponse::Metas`][]: <https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/requests/defineCatalogHandler.md#returns>
/// - [`ResourceResponse::MetasDetailed`][]: None
/// - [`ResourceResponse::Meta`][]: <https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/requests/defineMetaHandler.md#returns>
/// - [`ResourceResponse::Addons`][]: <https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/requests/defineResourceHandler.md#returns>
/// - [`ResourceResponse::Streams`][]: <https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/requests/defineStreamHandler.md#returns>
/// - [`ResourceResponse::Subtitles`][]: <https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/requests/defineSubtitlesHandler.md#returns>
///
///
/// # Examples
///
/// ```
/// use serde_json::json;
///
/// use stremio_core::types::{
///     addon::{ResourceResponseCache, ResourceResponse},
///     resource::{Stream, StreamSource, StreamBehaviorHints},
/// };
///
/// let cache_info_json = json!({
///     "streams": [
///         {
///             "name": "Addon\n4k",
///             "title": "South Park - Seasons 1 to 25 (S01-S25) Collectors Edition The Movie and Extras [NVEnc 10Bit 1080p to 2160p HEVC][DD DDP & TrueHD 5.1Ch]\nSeason 02/South Park - S02E05 - Conjoined Fetus Lady.mp4",
///             "url": "https://example-url-stream.com/South_Park_S02_E05.mp4",
///         },
///     ],
///     "cacheMaxAge": 3600,
///     "staleRevalidate": 14400,
///     "staleError": 604800,
///
/// });
///
/// let response_cache = serde_json::from_value(cache_info_json).expect("Should deserialize");
/// assert_eq!(ResourceResponseCache {
///     cache_max_age: Some(3600),
///     stale_revalidate: Some(14400),
///     stale_error: Some(604800),
///     resource: ResourceResponse::Streams{
///         streams: vec![
///             Stream {
///                 source: StreamSource::Url { url: "https://example-url-stream.com/South_Park_S02_E05.mp4".parse().unwrap() },
///                 name: Some("Addon\n4k".into()),
///                 description: Some("South Park - Seasons 1 to 25 (S01-S25) Collectors Edition The Movie and Extras [NVEnc 10Bit 1080p to 2160p HEVC][DD DDP & TrueHD 5.1Ch]\nSeason 02/South Park - S02E05 - Conjoined Fetus Lady.mp4".into()),
///                 thumbnail: None,
///                 subtitles: vec![],
///                 behavior_hints: StreamBehaviorHints::default(),
///             }
///         ]
///     }
/// }, response_cache);
/// ```

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Clone, TryInto, Serialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
#[serde_as]
pub enum ResourceResponse {
    /// [`MetaItemPreview`] response without the `videos` key of an item
    ///
    /// # Examples
    ///
    /// ```
    /// use stremio_core::types::addon::ResourceResponse;
    ///
    /// let null_metas = serde_json::json!({ "metas": null });
    /// let null_metas = serde_json::from_value::<ResourceResponse>(null_metas).expect("Should be valid");
    ///
    /// match null_metas {
    ///     ResourceResponse::Metas { metas } => assert!(metas.is_empty()),
    ///     _ => panic!("Whoops, wrong variant!"),
    /// }
    /// ```
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
        addons: Vec<Descriptor>,
    },
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct SkipError<T: for<'a> Deserialize<'a> + Serialize>(
    #[serde_as(as = "VecSkipError<_>")] Vec<T>,
);

impl<'de> Deserialize<'de> for ResourceResponse {
    /// Custom deserialize for ResourceResponse which expects only 1 of the required fields
    /// to be set in the response object
    fn deserialize<D>(deserializer: D) -> Result<ResourceResponse, D::Error>
    where
        D: Deserializer<'de>,
    {
        let keys = [
            "metas",
            "metasDetailed",
            "meta",
            "streams",
            "subtitles",
            "addons",
        ];

        let value =
            serde_path_to_error::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let mut value = match value {
            serde_json::Value::Object(value) => value,
            _ => {
                return Err(serde::de::Error::custom(
                    "Cannot deserialize as ResourceResponse, expected an Object response",
                ))
            }
        };

        // check whether we have one of the expected keys or if we have more than 1 which is not allowed!
        let unique_keys_count = keys.iter().filter(|key| value.contains_key(**key)).count();
        if unique_keys_count == 0 {
            return Err(serde::de::Error::custom(
                format!("Cannot deserialize as ResourceResponse, the expected Object response didn't contain any of the required keys: {}",
                keys.join(", ")),
            ));
        }
        if unique_keys_count > 1 {
            return Err(serde::de::Error::custom(
                format!("Cannot deserialize as ResourceResponse, the expected Object response contained more than 1 of the unique keys: {}",
                keys.join(", ")),
            ));
        }

        if let Some(value) = value.get_mut("metas") {
            Ok(ResourceResponse::Metas {
                metas: match value.take() {
                    serde_json::Value::Null => Ok(vec![]),
                    value => serde_json::from_value::<SkipError<_>>(value).map(|skip| skip.0),
                }
                .map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("metasDetailed") {
            Ok(ResourceResponse::MetasDetailed {
                metas_detailed: match value.take() {
                    serde_json::Value::Null => Ok(vec![]),
                    value => serde_json::from_value::<SkipError<_>>(value).map(|skip| skip.0),
                }
                .map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("meta") {
            Ok(ResourceResponse::Meta {
                meta: serde_json::from_value(value.take()).map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("streams") {
            Ok(ResourceResponse::Streams {
                streams: match value.take() {
                    serde_json::Value::Null => Ok(vec![]),
                    value => serde_json::from_value::<SkipError<_>>(value).map(|skip| skip.0),
                }
                .map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("subtitles") {
            Ok(ResourceResponse::Subtitles {
                subtitles: match value.take() {
                    serde_json::Value::Null => Ok(vec![]),
                    value => serde_json::from_value::<SkipError<_>>(value).map(|skip| skip.0),
                }
                .map_err(serde::de::Error::custom)?,
            })
        } else if let Some(value) = value.get_mut("addons") {
            Ok(ResourceResponse::Addons {
                addons: match value.take() {
                    serde_json::Value::Null => Ok(vec![]),
                    value => serde_json::from_value::<SkipError<_>>(value).map(|skip| skip.0),
                }
                .map_err(serde::de::Error::custom)?,
            })
        } else {
            // we should never get to this else, as we already check for missing required key
            // but we're leaving it to remove the danger of a developer forgetting to add a new key to the list.
            Err(serde::de::Error::custom(
                format!("Cannot deserialize as ResourceResponse, the expected Object response didn't contain any of the required keys: {}",
                keys.join(", ")
            )
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_value;

    use super::*;

    #[test]
    fn test_response_deserialization_keys() {
        // Bad json, should trigger the serde_error_to_path
        {
            // object key must be a string, number provided
            let json_response = r#"{
                "some_key": {
                    "valid_value": 9999999999999999999999999,
                    5
                }
            }"#;
            let result_err = serde_json::from_str::<ResourceResponse>(json_response)
                .expect_err("Should be an error");

            assert_eq!(
                result_err.to_string(),
                "some_key.?: key must be a string at line 4 column 21"
            );
            assert_eq!(4, result_err.line());
            assert_eq!(21, result_err.column());
        }

        // Wrong ResourceResponse, not an object response
        {
            let json_response = serde_json::json!(256);
            let result = from_value::<ResourceResponse>(json_response);

            assert!(
                result
                    .expect_err("Should be an error")
                    .to_string()
                    .contains("expected an Object response"),
                "Message does not include the text 'expected an Object response'"
            );
        }

        // Wrong ResourceResponse, missing a required key, i.e. non-existing variant
        {
            let json_response = serde_json::json!({
                "unknownVariant": {"test": 1}
            });
            let result = from_value::<ResourceResponse>(json_response);

            assert!(
                result
                    .expect_err("Should be an error")
                    .to_string()
                    .contains("didn't contain any of the required keys"),
                "Message does not include the text 'didn't contain any of the required keys'"
            );
        }

        // Wrong ResourceResponse, multiple exclusive keys, i.e. bad variant values
        {
            let json_response = serde_json::json!({
                "metas": {},
                "metasDetailed": {},
            });
            let result = from_value::<ResourceResponse>(json_response);

            assert!(
                result.expect_err("Should be an error").to_string().contains("Object response contained more than 1 of the unique keys"),
                "Message does not include the text 'Object response contained more than 1 of the unique keys'"
            );
        }
        // Wrong ResourceResponse, invalid type, expected sequence (Vec) got map (Object)
        {
            let json_response = serde_json::json!({
                "metas": {"object_key": "value"}
            });
            let result = from_value::<ResourceResponse>(json_response);

            assert!(
                result
                    .expect_err("Should be an error")
                    .to_string()
                    .contains("invalid type: map, expected a sequence"),
                "Message does not include the text 'invalid type: map, expected a sequence'"
            );
        }
    }
}
