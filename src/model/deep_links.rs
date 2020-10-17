use flate2::write::ZlibEncoder;
use flate2::Compression;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde::Serialize;
use std::borrow::Borrow;
use std::io;
use std::io::Write;
use stremio_core::models::library_with_filters::Sort;
use stremio_core::types::addon::{ExtraValue, ResourceRequest};
use stremio_core::types::library::LibraryItem;
use stremio_core::types::resource::{MetaItem, MetaItemPreview, Stream, Video};
use url::form_urlencoded;

const URI_COMPONENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

#[derive(Serialize)]
pub struct LibraryItemDeepLinks {
    meta_details_videos: Option<String>,
    meta_details_streams: Option<String>,
    player: Option<String>,
}

impl From<&LibraryItem> for LibraryItemDeepLinks {
    fn from(item: &LibraryItem) -> Self {
        LibraryItemDeepLinks {
            meta_details_videos: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .cloned()
                .xor(Some(format!(
                    "#/metadetails/{}/{}",
                    utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET)
                ))),
            meta_details_streams: item
                .state
                .video_id
                .as_ref()
                .or_else(|| item.behavior_hints.default_video_id.as_ref())
                .map(|video_id| {
                    format!(
                        "#/metadetails/{}/{}/{}",
                        utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&video_id, URI_COMPONENT_ENCODE_SET)
                    )
                }),
            player: None, // TODO use StreamsBucket
        }
    }
}

#[derive(Serialize)]
pub struct MetaItemDeepLinks {
    meta_details_videos: Option<String>,
    meta_details_streams: Option<String>,
}

impl From<&MetaItemPreview> for MetaItemDeepLinks {
    fn from(item: &MetaItemPreview) -> Self {
        MetaItemDeepLinks {
            meta_details_videos: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .cloned()
                .xor(Some(format!(
                    "#/metadetails/{}/{}",
                    utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET)
                ))),
            meta_details_streams: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .map(|video_id| {
                    format!(
                        "#/metadetails/{}/{}/{}",
                        utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&video_id, URI_COMPONENT_ENCODE_SET)
                    )
                }),
        }
    }
}

impl From<&MetaItem> for MetaItemDeepLinks {
    fn from(item: &MetaItem) -> Self {
        MetaItemDeepLinks {
            meta_details_videos: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .cloned()
                .xor(Some(format!(
                    "#/metadetails/{}/{}",
                    utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET)
                ))),
            meta_details_streams: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .map(|video_id| {
                    format!(
                        "#/metadetails/{}/{}/{}",
                        utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&video_id, URI_COMPONENT_ENCODE_SET)
                    )
                }),
        }
    }
}

#[derive(Serialize)]
pub struct VideoDeepLinks {
    meta_details_streams: String,
    player: Option<String>,
}

impl From<(&Video, &ResourceRequest)> for VideoDeepLinks {
    fn from((video, request): (&Video, &ResourceRequest)) -> Self {
        VideoDeepLinks {
            meta_details_streams: format!(
                "#/metadetails/{}/{}/{}",
                utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&video.id, URI_COMPONENT_ENCODE_SET)
            ),
            player: video
                .streams
                .first()
                .filter(|_| video.streams.len() == 1)
                .map(|stream| {
                    format!(
                        "#/player/{}/{}/{}/{}/{}/{}",
                        utf8_percent_encode(
                            &base64::encode(
                                gz_encode(serde_json::to_string(stream).unwrap()).unwrap()
                            ),
                            URI_COMPONENT_ENCODE_SET
                        ),
                        utf8_percent_encode(&request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&video.id, URI_COMPONENT_ENCODE_SET)
                    )
                }),
        }
    }
}

#[derive(Serialize)]
pub struct StreamDeepLinks {
    player: String,
}

impl From<&Stream> for StreamDeepLinks {
    fn from(stream: &Stream) -> Self {
        StreamDeepLinks {
            player: format!(
                "#/player/{}",
                utf8_percent_encode(
                    &base64::encode(gz_encode(serde_json::to_string(stream).unwrap()).unwrap()),
                    URI_COMPONENT_ENCODE_SET
                ),
            ),
        }
    }
}

impl From<(&Stream, &ResourceRequest, &ResourceRequest)> for StreamDeepLinks {
    fn from(
        (stream, stream_request, meta_request): (&Stream, &ResourceRequest, &ResourceRequest),
    ) -> Self {
        StreamDeepLinks {
            player: format!(
                "#/player/{}/{}/{}/{}/{}/{}",
                utf8_percent_encode(
                    &base64::encode(gz_encode(serde_json::to_string(stream).unwrap()).unwrap()),
                    URI_COMPONENT_ENCODE_SET
                ),
                utf8_percent_encode(&stream_request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&meta_request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&meta_request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&meta_request.path.id, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&stream_request.path.id, URI_COMPONENT_ENCODE_SET)
            ),
        }
    }
}

#[derive(Serialize)]
pub struct MetaCatalogResourceDeepLinks {
    discover: String,
}

impl From<&ResourceRequest> for MetaCatalogResourceDeepLinks {
    fn from(request: &ResourceRequest) -> Self {
        MetaCatalogResourceDeepLinks {
            discover: format!(
                "#/discover/{}/{}/{}?{}",
                utf8_percent_encode(&request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
                query_params_encode(
                    request
                        .path
                        .extra
                        .iter()
                        .map(|ExtraValue { name, value }| (name, value))
                )
            ),
        }
    }
}

#[derive(Serialize)]
pub struct LibraryDeepLinks {
    library: String,
}

impl From<&String> for LibraryDeepLinks {
    fn from(root: &String) -> Self {
        LibraryDeepLinks {
            library: format!("#/{}", root),
        }
    }
}

impl From<(&String, &Option<String>, &Sort)> for LibraryDeepLinks {
    fn from((root, r#type, sort): (&String, &Option<String>, &Sort)) -> Self {
        LibraryDeepLinks {
            library: match r#type {
                Some(r#type) => format!(
                    "#/{}/{}?{}",
                    root,
                    utf8_percent_encode(r#type, URI_COMPONENT_ENCODE_SET),
                    query_params_encode(&[("sort", serde_json::to_string(sort).unwrap())])
                ),
                _ => format!(
                    "#/{}?{}",
                    root,
                    query_params_encode(&[("sort", serde_json::to_string(sort).unwrap())])
                ),
            },
        }
    }
}

fn gz_encode(value: String) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(value.as_bytes())?;
    Ok(encoder.finish()?)
}

fn query_params_encode<I, K, V>(query_params: I) -> String
where
    I: IntoIterator,
    I::Item: Borrow<(K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    form_urlencoded::Serializer::new(String::new())
        .extend_pairs(query_params)
        .finish()
}
