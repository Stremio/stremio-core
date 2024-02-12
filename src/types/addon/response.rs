use derive_more::TryInto;
use serde::{de::Deserializer, Deserialize, Serialize};
use serde_with::{serde_as, VecSkipError};
use url::Url;

use crate::types::{
    addon::DescriptorPreview,
    resource::{MetaItem, MetaItemPreview, Stream, Subtitles},
};

/// Resource Response from an addon.
///
/// Deserializing the struct from json will skip any invalid Vec items
/// and will skip any unknown to the variants fields.
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
}

impl ResourceResponse {
    /// Convert any relative path in `Stream.source` with absolute url using the provided addon's transport url
    pub fn convert_relative_paths(&mut self, addon_transport_url: Url) {
        match self {
            ResourceResponse::Metas { ref mut metas } => {
                metas
                    .iter_mut()
                    .flat_map(|meta_item_preview| {
                        meta_item_preview
                            .trailer_streams
                            .iter_mut()
                            .filter_map(|stream| stream.with_addon_url(&addon_transport_url).ok())
                        // .collect::<Result<_, _>>()
                    })
                    .collect()
            }
            ResourceResponse::MetasDetailed {
                ref mut metas_detailed,
            } => {
                metas_detailed
                    .iter_mut()
                    .flat_map(|meta_item| {
                        // MetaItem videos
                        meta_item
                            .videos
                            .iter_mut()
                            .flat_map(|video| {
                                // MetaItem video streams
                                video
                                    .streams
                                    .iter_mut()
                                    .filter_map(|stream| {
                                        stream.with_addon_url(&addon_transport_url).ok()
                                    })
                                    .chain(
                                        // MetaItem videos' trailer streams
                                        video.trailer_streams.iter_mut().filter_map(|stream| {
                                            stream.with_addon_url(&addon_transport_url).ok()
                                        }),
                                    )
                            })
                            // Trailer Streams of the MetaItemPreview
                            .chain(meta_item.preview.trailer_streams.iter_mut().filter_map(
                                |stream| stream.with_addon_url(&addon_transport_url).ok(),
                            ))
                    })
                    .collect()
            }
            ResourceResponse::Meta { meta } => meta
                .videos
                .iter_mut()
                .flat_map(|video| {
                    // MetaItem video streams
                    video
                        .streams
                        .iter_mut()
                        .filter_map(|stream| stream.with_addon_url(&addon_transport_url).ok())
                        .chain(
                            // MetaItem videos' trailer streams
                            video.trailer_streams.iter_mut().filter_map(|stream| {
                                stream.with_addon_url(&addon_transport_url).ok()
                            }),
                        )
                })
                // Trailer Streams of the MetaItemPreview
                .chain(
                    meta.preview
                        .trailer_streams
                        .iter_mut()
                        .filter_map(|stream| stream.with_addon_url(&addon_transport_url).ok()),
                )
                .collect(),
            ResourceResponse::Streams { streams } => streams
                .iter_mut()
                .filter_map(|stream| stream.with_addon_url(&addon_transport_url).ok())
                .collect(),
            ResourceResponse::Subtitles { subtitles } => subtitles
                .iter_mut()
                .filter_map(|subtitle| subtitle.url.with_addon_url(&addon_transport_url).ok())
                .collect(),
            ResourceResponse::Addons { .. } => {
                // for addons - do nothing
            }
        }
    }
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
        } else {
            Err(serde::de::Error::custom(
                "Cannot deserialize as ResourceResponse",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use url::Url;

    use crate::types::resource::{
        MetaItem, MetaItemPreview, Stream, StreamBehaviorHints, StreamSource, UrlExtended, Video,
    };

    use super::ResourceResponse;

    pub static ADDON_TRANSPORT_URL: Lazy<Url> =
        Lazy::new(|| "https://example-addon.com/manifest.json".parse().unwrap());

    #[test]
    fn replace_relative_path_for_meta() {
        let relative_stream = Stream {
            source: StreamSource::Url {
                url: UrlExtended::RelativePath("/stream/path/tt123456.json".into()),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        };

        let relative_video_trailer_stream = Stream {
            source: StreamSource::Url {
                url: UrlExtended::RelativePath("/stream/video/trailer/path/tt123456:1.json".into()),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        };

        let relative_trailer_stream = Stream {
            source: StreamSource::Url {
                url: UrlExtended::RelativePath("/stream/trailer/path/tt123456.json".into()),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        };

        let relative_meta_preview = {
            let mut preview = MetaItemPreview::default();
            preview
                .trailer_streams
                .push(relative_trailer_stream.clone());
            preview
        };

        let relative_video_stream = {
            let mut video = Video::default();
            video
                .trailer_streams
                .push(relative_video_trailer_stream.clone());

            video.streams.push(relative_stream.clone());
            video
        };

        // Meta response with relative path
        {
            let mut resource_response = ResourceResponse::Meta {
                meta: MetaItem {
                    preview: relative_meta_preview.clone(),
                    videos: vec![relative_video_stream.clone()],
                },
            };

            resource_response.convert_relative_paths(ADDON_TRANSPORT_URL.clone());

            let meta = match resource_response {
                ResourceResponse::Meta { meta } => meta,
                _ => unreachable!(),
            };
            // Meta trailer stream
            assert_eq!(
                "https://example-addon.com/stream/trailer/path/tt123456.json",
                &meta
                    .preview
                    .trailer_streams
                    .first()
                    .unwrap()
                    .download_url()
                    .unwrap()
            );

            // Video stream
            assert_eq!(
                "https://example-addon.com/stream/path/tt123456.json",
                &meta
                    .videos
                    .first()
                    .unwrap()
                    .streams
                    .first()
                    .unwrap()
                    .download_url()
                    .unwrap()
            );

            // Video trailer stream

            assert_eq!(
                "https://example-addon.com/stream/video/trailer/path/tt123456:1.json",
                &meta
                    .videos
                    .first()
                    .unwrap()
                    .trailer_streams
                    .first()
                    .unwrap()
                    .download_url()
                    .unwrap()
            );
        }
    }
}
