use crate::constants::{BASE64, URI_COMPONENT_ENCODE_SET, YOUTUBE_ADDON_ID_PREFIX};
use crate::types::resource::Subtitles;
use base64::Engine;
use boolinator::Boolinator;
#[cfg(test)]
use derivative::Derivative;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use magnet_url::Magnet;
use percent_encoding::utf8_percent_encode;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DefaultOnNull};
use std::collections::HashMap;
use std::io::Write;
use stremio_serde_hex::{SerHex, Strict};
use url::{form_urlencoded, Url};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    #[serde(flatten)]
    pub source: StreamSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(alias = "title", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtitles: Vec<Subtitles>,
    #[serde(default, skip_serializing_if = "is_default_value")]
    pub behavior_hints: StreamBehaviorHints,
}

impl Stream {
    pub fn magnet_url(&self) -> Option<Magnet> {
        match &self.source {
            StreamSource::Url { url } if url.scheme() == "magnet" => Magnet::new(url.as_str()).ok(),
            StreamSource::Torrent {
                info_hash,
                announce,
                ..
            } => Some(Magnet {
                dn: self.name.to_owned(),
                hash_type: Some("btih".to_string()),
                xt: Some(hex::encode(info_hash)),
                xl: None,
                tr: announce
                    .iter()
                    // `tracker` and `dht` prefixes are used internally by the server.js
                    // we need to remove those prefixes when generating the magnet URL
                    .map(|tracker| {
                        tracker
                            .strip_prefix("tracker:")
                            .map(ToString::to_string)
                            .unwrap_or_else(|| tracker.to_owned())
                    })
                    .map(|tracker| {
                        tracker
                            .strip_prefix("dht:")
                            .map(ToString::to_string)
                            .unwrap_or_else(|| tracker.to_owned())
                    })
                    .map(|tracker| {
                        utf8_percent_encode(&tracker, URI_COMPONENT_ENCODE_SET).to_string()
                    })
                    .collect::<Vec<String>>(),
                kt: None,
                ws: None,
                acceptable_source: None,
                mt: None,
                xs: None,
            }),
            _ => None,
        }
    }
    pub fn encode(&self) -> Result<String, anyhow::Error> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::none());
        let stream = serde_json::to_string(&self)?;
        encoder.write_all(stream.as_bytes())?;
        let stream = encoder.finish()?;
        let stream = BASE64.encode(stream);
        Ok(stream)
    }
    pub fn decode(stream: String) -> Result<Self, anyhow::Error> {
        let stream = BASE64.decode(stream)?;
        let mut writer = Vec::new();
        let mut decoder = ZlibDecoder::new(writer);
        decoder.write_all(&stream)?;
        writer = decoder.finish()?;
        let stream = String::from_utf8(writer)?;
        let stream = serde_json::from_str(&stream)?;
        Ok(stream)
    }
    pub fn youtube(video_id: &str) -> Option<Self> {
        video_id
            .starts_with(YOUTUBE_ADDON_ID_PREFIX)
            .as_option()
            // video id is in format: yt_id:YT_CHANNEL_ID:YT_VIDEO_ID
            .and_then(|_| video_id.split(':').nth(2))
            .map(|yt_id| Self {
                source: StreamSource::YouTube {
                    yt_id: yt_id.to_owned(),
                },
                name: None,
                description: None,
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: Default::default(),
            })
    }

    pub fn download_url(&self) -> Option<String> {
        match &self.source {
            StreamSource::Url { url } if url.scheme() == "magnet" => {
                self.magnet_url().map(|magnet_url| magnet_url.to_string())
            }
            StreamSource::Url { url } => Some(url.to_string()),
            StreamSource::Rar {
                rar_urls,
                file_idx,
                file_must_include,
            } => todo!(),
            StreamSource::Zip {
                zip_urls,
                file_idx,
                file_must_include,
            } => todo!(),
            StreamSource::Torrent { .. } => {
                self.magnet_url().map(|magnet_url| magnet_url.to_string())
            }
            StreamSource::YouTube { .. } => self.youtube_url(),
            StreamSource::External { external_url, .. } => {
                external_url.as_ref().map(|url| url.to_string())
            }
            StreamSource::PlayerFrame { player_frame_url } => Some(player_frame_url.to_string()),
        }
    }

    pub fn m3u_data_uri(&self, streaming_server_url: Option<&Url>) -> Option<String> {
        self.streaming_url(streaming_server_url).map(|url| {
            format!(
                "data:application/octet-stream;charset=utf-8;base64,{}",
                BASE64.encode(format!("#EXTM3U\n#EXTINF:0\n{url}"))
            )
        })
    }
    pub fn streaming_url(&self, streaming_server_url: Option<&Url>) -> Option<String> {
        match (&self.source, streaming_server_url) {
            (StreamSource::Url { url }, streaming_server_url) if url.scheme() != "magnet" => {
                // If proxy headers are set and streaming server is available, build the proxied streaming url from streaming server url
                // Otherwise return the url
                match (&self.behavior_hints.proxy_headers, streaming_server_url) {
                    (
                        Some(StreamProxyHeaders { request, response }),
                        Some(streaming_server_url),
                    ) => {
                        let mut streaming_url = streaming_server_url.to_owned();
                        let mut proxy_query = form_urlencoded::Serializer::new(String::new());
                        let origin = format!("{}://{}", url.scheme(), url.authority());
                        proxy_query.append_pair("d", origin.as_str());
                        proxy_query.extend_pairs(
                            request
                                .iter()
                                .map(|header| ("h", format!("{}:{}", header.0, header.1))),
                        );
                        proxy_query.extend_pairs(
                            response
                                .iter()
                                .map(|header| ("r", format!("{}:{}", header.0, header.1))),
                        );
                        streaming_url
                            .path_segments_mut()
                            .ok()?
                            .push("proxy")
                            .push(proxy_query.finish().as_str())
                            .push(&url.path()[1..]);
                        streaming_url.set_query(url.query());
                        Some(streaming_url.to_string())
                    }
                    _ => Some(url.to_string()),
                }
            }
            (
                StreamSource::Torrent {
                    info_hash,
                    file_idx,
                    announce,
                    file_must_include,
                },
                Some(streaming_server_url),
            ) => {
                let mut url = streaming_server_url.to_owned();
                match url.path_segments_mut() {
                    Ok(mut path) => {
                        path.extend([
                            &hex::encode(info_hash),
                            // When fileIndex is not provided use -1, which will tell the
                            // streaming server to choose the file with the largest size from the torrent
                            &file_idx.map_or_else(|| "-1".to_string(), |idx| idx.to_string()),
                        ]);
                    }
                    _ => return None,
                }

                let mut query = vec![];
                if !announce.is_empty() {
                    query.extend(announce.iter().map(|tracker| ("tr", tracker)));
                }

                match file_must_include {
                    Some(file_must_include) if !file_must_include.is_empty() => {
                        query.push(("f", file_must_include));
                    }
                    _ => {}
                }

                url.query_pairs_mut().extend_pairs(query);

                Some(url.to_string())
            }
            (
                StreamSource::Zip {
                    zip_urls,
                    file_idx,
                    file_must_include,
                },
                Some(streaming_server_url),
            ) => {
                todo!()
            }
            (
                StreamSource::Rar {
                    rar_urls,
                    file_idx,
                    file_must_include,
                },
                Some(streaming_server_url),
            ) => {
                // Url::join should never happen.
                // let mut rar_url = streaming_server_url.join("rar/stream").ok()?;
                // struct StreamQueryParams {
                //     key:
                // }
                // rar_url.set_query(Some(serde_url_params::to_string()))

                // Some(rar_url.to_string())

                todo!()
            }
            (StreamSource::YouTube { yt_id }, Some(streaming_server_url)) => {
                let mut url = streaming_server_url.to_owned();
                match url.path_segments_mut() {
                    Ok(mut path) => {
                        path.push("yt");
                        path.push(
                            &utf8_percent_encode(yt_id, URI_COMPONENT_ENCODE_SET).to_string(),
                        );
                    }
                    _ => return None,
                };
                Some(url.to_string())
            }
            _ => None,
        }
    }
    pub fn youtube_url(&self) -> Option<String> {
        match &self.source {
            StreamSource::YouTube { yt_id } => Some(format!(
                "https://youtube.com/watch?v={}",
                utf8_percent_encode(yt_id, URI_COMPONENT_ENCODE_SET)
            )),
            _ => None,
        }
    }

    #[inline]
    pub fn is_source_match(&self, other_stream: &Stream) -> bool {
        self.source == other_stream.source
    }

    #[inline]
    pub fn is_binge_match(&self, other_stream: &Stream) -> bool {
        match (
            &self.behavior_hints.binge_group,
            &other_stream.behavior_hints.binge_group,
        ) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

///
/// # Examples
///
/// Stream source Url
///
/// [`StreamSource::Rar`] with `rarUrls` field:
///
/// ```
/// use stremio_core::types::resource::StreamSource;
///
/// let streams_json = serde_json::json!([
/// {
///     "rarUrls": ["https://example-source.com/file.rar", "https://example-source2.com/file2.rar"],
///     // ...Stream
/// },
/// {
///     "rarUrls": ["https://example-source3.com/file.rar", "https://example-source4.com/file2.rar"],
///     "fileIdx": 1,
///     "fileMustInclude": "includeFile1",
///     // ...Stream
/// },
/// {
///     "rarUrls": ["https://example-source5.com/file.rar", "https://example-source6.com/file2.rar"],
///     "fileMustInclude": "includeFile2",
///     // ...Stream
/// },
/// {
///     "rarUrls": ["https://example-source7.com/file.rar", "https://example-source8.com/file2.rar"],
///     "fileIdx": 2,
///     // ...Stream
/// }
/// ]);
///
/// let expected = vec![
///     StreamSource::Rar {
///         rar_urls: vec!["https://example-source.com/file.rar".parse().unwrap(), "https://example-source2.com/file2.rar".parse().unwrap()],
///         file_idx: None,
///         file_must_include: None,
///     },
///     StreamSource::Rar {
///         rar_urls: vec!["https://example-source3.com/file.rar".parse().unwrap(), "https://example-source4.com/file2.rar".parse().unwrap()],
///         file_idx: Some(1),
///         file_must_include: Some("includeFile1".into())
///     },
///     StreamSource::Rar {
///         rar_urls: vec!["https://example-source5.com/file.rar".parse().unwrap(), "https://example-source6.com/file2.rar".parse().unwrap()],
///         file_idx: None,
///         file_must_include: Some("includeFile2".into())
///     },
///     StreamSource::Rar {
///         rar_urls: vec!["https://example-source7.com/file.rar".parse().unwrap(), "https://example-source8.com/file2.rar".parse().unwrap()],
///         file_idx: Some(2),
///         file_must_include: None,
///     },
/// ];
///
/// let streams: Vec<StreamSource> = serde_json::from_value(streams_json).expect("Deserialize all StreamSources");
///
/// pretty_assertions::assert_eq!(streams, expected);
/// ```
///
/// [`StreamSource::Zip`] with `zipUrls` field:
///
/// ```
/// use stremio_core::types::resource::StreamSource;
///
/// let streams_json = serde_json::json!([
/// {
///     "zipUrls": ["https://example-source.com/file.rar", "https://example-source2.com/file2.rar"],
///     // ...Stream
/// },
/// {
///     "zipUrls": ["https://example-source3.com/file.rar", "https://example-source4.com/file2.rar"],
///     "fileIdx": 1,
///     "fileMustInclude": "includeFile1",
///     // ...Stream
/// },
/// {
///     "zipUrls": ["https://example-source5.com/file.rar", "https://example-source6.com/file2.rar"],
///     "fileMustInclude": "includeFile2",
///     // ...Stream
/// },
/// {
///     "zipUrls": ["https://example-source7.com/file.rar", "https://example-source8.com/file2.rar"],
///     "fileIdx": 2,
///     // ...Stream
/// }
/// ]);
///
/// let expected = vec![
///     StreamSource::Zip {
///         zip_urls: vec!["https://example-source.com/file.rar".parse().unwrap(), "https://example-source2.com/file2.rar".parse().unwrap()],
///         file_idx: None,
///         file_must_include: None,
///     },
///     StreamSource::Zip {
///         zip_urls: vec!["https://example-source3.com/file.rar".parse().unwrap(), "https://example-source4.com/file2.rar".parse().unwrap()],
///         file_idx: Some(1),
///         file_must_include: Some("includeFile1".into())
///     },
///     StreamSource::Zip {
///         zip_urls: vec!["https://example-source5.com/file.rar".parse().unwrap(), "https://example-source6.com/file2.rar".parse().unwrap()],
///         file_idx: None,
///         file_must_include: Some("includeFile2".into())
///     },
///     StreamSource::Zip {
///         zip_urls: vec!["https://example-source7.com/file.rar".parse().unwrap(), "https://example-source8.com/file2.rar".parse().unwrap()],
///         file_idx: Some(2),
///         file_must_include: None,
///     },
/// ];
///
/// let streams: Vec<StreamSource> = serde_json::from_value(streams_json).expect("Deserialize all StreamSources");
///
/// pretty_assertions::assert_eq!(streams, expected);
/// ```
#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(untagged)]
pub enum StreamSource {
    Url {
        url: Url,
    },
    #[cfg_attr(test, derivative(Default))]
    #[serde(rename_all = "camelCase")]
    YouTube {
        yt_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Rar {
        rar_urls: Vec<Url>,
        #[serde(default)]
        file_idx: Option<u16>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_must_include: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Zip {
        zip_urls: Vec<Url>,
        #[serde(default)]
        file_idx: Option<u16>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_must_include: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Torrent {
        #[serde(with = "SerHex::<Strict>")]
        info_hash: [u8; 20],
        file_idx: Option<u16>,
        #[serde_as(deserialize_as = "DefaultOnNull")]
        #[serde(default, alias = "sources")]
        announce: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_must_include: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    PlayerFrame {
        player_frame_url: Url,
    },
    #[serde(
        rename_all = "camelCase",
        deserialize_with = "deserialize_stream_source_external"
    )]
    External {
        #[serde(skip_serializing_if = "Option::is_none")]
        external_url: Option<Url>,
        #[serde(skip_serializing_if = "Option::is_none")]
        android_tv_url: Option<Url>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tizen_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        webos_url: Option<String>,
    },
}

type ExternalStreamSource = (Option<Url>, Option<Url>, Option<String>, Option<String>);

fn deserialize_stream_source_external<'de, D>(
    deserializer: D,
) -> Result<ExternalStreamSource, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Helper {
        external_url: Option<Url>,
        android_tv_url: Option<Url>,
        tizen_url: Option<String>,
        webos_url: Option<String>,
    }
    let source = Helper::deserialize(deserializer)?;
    if source.external_url.is_none()
        && source.android_tv_url.is_none()
        && source.tizen_url.is_none()
        && source.webos_url.is_none()
    {
        return Err(D::Error::custom("Invalid StreamSource::External"));
    };
    Ok((
        source.external_url,
        source.android_tv_url,
        source.tizen_url,
        source.webos_url,
    ))
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamProxyHeaders {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub request: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub response: HashMap<String, String>,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamBehaviorHints {
    #[serde(default, skip_serializing_if = "is_default_value")]
    pub not_web_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binge_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_whitelist: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_headers: Option<StreamProxyHeaders>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

fn is_default_value<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}
