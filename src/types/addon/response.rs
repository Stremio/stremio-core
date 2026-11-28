use std::{fmt, marker::PhantomData};

use derive_more::TryInto;
use serde::{
    de::{
        self, value::SeqAccessDeserializer, DeserializeSeed, Deserializer, IgnoredAny, MapAccess,
        SeqAccess, Unexpected, Visitor,
    },
    Deserialize, Serialize,
};
use serde_with::{serde_as, DeserializeAs, Same, VecSkipError};

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

const RESPONSE_KEYS: [&str; 6] = [
    "metas",
    "metasDetailed",
    "meta",
    "streams",
    "subtitles",
    "addons",
];

#[derive(PartialEq, Eq)]
enum ResponseKey {
    Metas,
    MetasDetailed,
    Meta,
    Streams,
    Subtitles,
    Addons,
}

struct ResponseKeySeed;

impl<'de> DeserializeSeed<'de> for ResponseKeySeed {
    type Value = Option<ResponseKey>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for ResponseKeySeed {
    type Value = Option<ResponseKey>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, key: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(match key {
            "metas" => Some(ResponseKey::Metas),
            "metasDetailed" => Some(ResponseKey::MetasDetailed),
            "meta" => Some(ResponseKey::Meta),
            "streams" => Some(ResponseKey::Streams),
            "subtitles" => Some(ResponseKey::Subtitles),
            "addons" => Some(ResponseKey::Addons),
            _ => None,
        })
    }
}

struct IgnoredValue;

impl<'de> DeserializeSeed<'de> for IgnoredValue {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IgnoredAny).map(drop)
    }
}

struct NullableVecSkipError<T>(PhantomData<T>);

impl<T> NullableVecSkipError<T> {
    const EXPECTING: &'static str = "a sequence";

    fn invalid(unexpected: Unexpected) -> Result<Vec<T>, String> {
        Err(<de::value::Error as de::Error>::invalid_type(unexpected, &Self::EXPECTING).to_string())
    }
}

impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for NullableVecSkipError<T> {
    type Value = Result<Vec<T>, String>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for NullableVecSkipError<T> {
    type Value = Result<Vec<T>, String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(Self::EXPECTING)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::invalid(Unexpected::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::invalid(Unexpected::Signed(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::invalid(Unexpected::Unsigned(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::invalid(Unexpected::Float(value)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::invalid(Unexpected::Str(value)))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::invalid(Unexpected::Bytes(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ok(vec![]))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ok(vec![]))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        VecSkipError::<Same>::deserialize_as(SeqAccessDeserializer::new(seq)).map(Ok)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while map.next_entry_seed(IgnoredValue, IgnoredValue)?.is_some() {}
        Ok(Self::invalid(Unexpected::Map))
    }
}

struct ResourceResponseVisitor;

impl<'de> Visitor<'de> for ResourceResponseVisitor {
    type Value = Result<ResourceResponse, String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an Object response")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut response: Option<(ResponseKey, Result<ResourceResponse, String>)> = None;
        while let Some(key) = map.next_key_seed(ResponseKeySeed)? {
            let Some(key) = key else {
                map.next_value_seed(IgnoredValue)?;
                continue;
            };
            if response
                .as_ref()
                .is_some_and(|(current, _)| *current != key)
            {
                return Err(de::Error::custom(format!(
                    "Cannot deserialize as ResourceResponse, the expected Object response contained more than 1 of the unique keys: {}",
                    RESPONSE_KEYS.join(", ")
                )));
            }
            let value = match key {
                ResponseKey::Metas => map
                    .next_value_seed(NullableVecSkipError(PhantomData))?
                    .map(|metas| ResourceResponse::Metas { metas }),
                ResponseKey::MetasDetailed => map
                    .next_value_seed(NullableVecSkipError(PhantomData))?
                    .map(|metas_detailed| ResourceResponse::MetasDetailed { metas_detailed }),
                ResponseKey::Meta => Ok(ResourceResponse::Meta {
                    meta: map.next_value()?,
                }),
                ResponseKey::Streams => map
                    .next_value_seed(NullableVecSkipError(PhantomData))?
                    .map(|streams| ResourceResponse::Streams { streams }),
                ResponseKey::Subtitles => map
                    .next_value_seed(NullableVecSkipError(PhantomData))?
                    .map(|subtitles| ResourceResponse::Subtitles { subtitles }),
                ResponseKey::Addons => map
                    .next_value_seed(NullableVecSkipError(PhantomData))?
                    .map(|addons| ResourceResponse::Addons { addons }),
            };
            response = Some((key, value));
        }
        Ok(match response {
            Some((_, response)) => response,
            None => Err(format!(
                "Cannot deserialize as ResourceResponse, the expected Object response didn't contain any of the required keys: {}",
                RESPONSE_KEYS.join(", ")
            )),
        })
    }
}

impl<'de> Deserialize<'de> for ResourceResponse {
    /// Custom deserialize for ResourceResponse which expects only 1 of the required fields
    /// to be set in the response object
    fn deserialize<D>(deserializer: D) -> Result<ResourceResponse, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut track = serde_path_to_error::Track::new();
        serde_path_to_error::Deserializer::new(deserializer, &mut track)
            .deserialize_map(ResourceResponseVisitor)
            .map_err(|error| {
                de::Error::custom(serde_path_to_error::Error::new(track.path(), error))
            })?
            .map_err(de::Error::custom)
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
