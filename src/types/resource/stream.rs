use core::fmt;
use std::{collections::HashMap, io::Write};

use itertools::Itertools;

use base64::Engine;
use boolinator::Boolinator;
use flate2::{
    write::{ZlibDecoder, ZlibEncoder},
    Compression,
};
use magnet_url::Magnet;
use percent_encoding::utf8_percent_encode;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DefaultOnNull, VecSkipError};
use url::{form_urlencoded, Url};

use stremio_serde_hex::{SerHex, Strict};

use crate::{
    constants::{BASE64, URI_COMPONENT_ENCODE_SET, YOUTUBE_ADDON_ID_PREFIX},
    types::{
        resource::Subtitles,
        streaming_server::{ArchiveStreamBody, ArchiveStreamOptions, FtpStreamBody},
    },
};

/// # Examples
///
/// ```
/// use stremio_core::types::resource::{Stream, StreamSource, StreamBehaviorHints};
///
/// let expected_stream = Stream {
///     source: StreamSource::Url { url: "https://example.com/some-awesome-video-file.mp4".parse().unwrap()},
///     name: None,
///     description: None,
///     thumbnail: None,
///     subtitles: vec![],
///     behavior_hints: StreamBehaviorHints::default(),
/// };
///
/// let default_fields_json = serde_json::json!({
///     "url": "https://example.com/some-awesome-video-file.mp4",
/// });
/// let default_fields = serde_json::from_value::<Stream>(default_fields_json).unwrap();
///
/// assert_eq!(default_fields, expected_stream);
///
/// let null_fields_json = serde_json::json!({
///     "url": "https://example.com/some-awesome-video-file.mp4",
///     "name": null,
///     "description": null,
///     "thumbnail": null,
///     "subtitles": null,
///     "behaviorHints": null,
/// });
///
/// let null_fields = serde_json::from_value::<Stream>(null_fields_json).unwrap();
///
/// assert_eq!(null_fields, expected_stream);
/// ```
#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    // pub struct Stream {
    #[serde(flatten)]
    pub source: StreamSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, alias = "title", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde_as(as = "DefaultOnNull<VecSkipError<_>>")]
    pub subtitles: Vec<Subtitles>,
    #[serde(default, skip_serializing_if = "is_default_value")]
    #[serde_as(as = "DefaultOnNull")]
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
            StreamSource::Rar { .. } => None,
            StreamSource::Zip7 { .. } => None,
            StreamSource::Zip { .. } => None,
            StreamSource::Tar { .. } => None,
            StreamSource::Tgz { .. } => None,
            StreamSource::Nzb { .. } => None,
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

    pub fn streaming_url(&self, streaming_server_url: Option<&Url>) -> Option<Url> {
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

                        streaming_url.set_path(&format!(
                            "proxy/{query}/{url_path}",
                            query = proxy_query.finish().as_str(),
                            url_path = &url.path().strip_prefix('/').unwrap_or(url.path()),
                        ));

                        streaming_url.set_query(url.query());
                        Some(streaming_url)
                    }
                    _ => Some(url.to_owned()),
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

                // setup query params
                {
                    let mut query_params = url.query_pairs_mut();

                    if !announce.is_empty() {
                        query_params.extend_pairs(
                            announce.iter().map(|tracker| ("tr", tracker.to_owned())),
                        );
                    }

                    if !file_must_include.is_empty() {
                        query_params.extend_pairs(
                            file_must_include
                                .iter()
                                .map(|file_must_include| ("f", file_must_include.to_owned())),
                        );
                    }
                }

                Some(url)
            }
            (
                StreamSource::Rar {
                    urls,
                    file_idx,
                    file_must_include,
                },
                Some(streaming_server_url),
            ) => {
                let query =
                    Self::archive_query(&urls, &file_idx, &file_must_include, streaming_server_url);

                let mut url = streaming_server_url.to_owned();
                url.set_path(&format!("rar/create/{query}", query = query.as_str(),));

                Some(url)
            }
            (
                StreamSource::Zip {
                    urls,
                    file_idx,
                    file_must_include,
                },
                Some(streaming_server_url),
            ) => {
                let query =
                    Self::archive_query(&urls, &file_idx, &file_must_include, streaming_server_url);

                let mut url = streaming_server_url.to_owned();
                url.set_path(&format!("zip/create/{query}", query = query.as_str(),));

                Some(url)
            }
            (
                StreamSource::Zip7 {
                    urls,
                    file_idx,
                    file_must_include,
                },
                Some(streaming_server_url),
            ) => {
                let query =
                    Self::archive_query(&urls, &file_idx, &file_must_include, streaming_server_url);

                let mut url = streaming_server_url.to_owned();
                url.set_path(&format!("7zip/create/{query}", query = query.as_str(),));

                Some(url)
            }
            (
                StreamSource::Tgz {
                    urls,
                    file_idx,
                    file_must_include,
                },
                Some(streaming_server_url),
            ) => {
                let query =
                    Self::archive_query(&urls, &file_idx, &file_must_include, streaming_server_url);

                let mut url = streaming_server_url.to_owned();
                url.set_path(&format!("tgz/create/{query}", query = query.as_str(),));

                Some(url)
            }
            (
                StreamSource::Tar {
                    urls,
                    file_idx,
                    file_must_include,
                },
                Some(streaming_server_url),
            ) => {
                let query =
                    Self::archive_query(&urls, &file_idx, &file_must_include, streaming_server_url);

                let mut url = streaming_server_url.to_owned();
                url.set_path(&format!("tar/create/{query}", query = query.as_str(),));

                Some(url)
            }
            (StreamSource::Nzb { nzb_url, servers }, Some(streaming_server_url)) => {
                let servers = servers
                    .into_iter()
                    .filter_map(|server_url| {
                        Self::ftp_url_handler(streaming_server_url, &server_url)
                    })
                    .collect_vec();

                let payload = StreamSource::Nzb {
                    nzb_url: Self::ftp_url_handler(streaming_server_url, &nzb_url)
                        .expect("Streaming server availability is already checked"),
                    servers,
                };

                let lz_param = serde_json::to_string(&payload).unwrap();
                let mut query = form_urlencoded::Serializer::new(String::new());
                query.append_pair("lz", &lz_str::compress_to_encoded_uri_component(&lz_param));

                let mut url = streaming_server_url.to_owned();
                url.set_path(&format!(
                    "nzb/create/{query}",
                    query = query.finish().as_str(),
                ));

                Some(url)
            }
            (StreamSource::YouTube { yt_id }, Some(streaming_server_url)) => {
                let mut url = streaming_server_url.to_owned();
                match url.path_segments_mut() {
                    Ok(mut path) => {
                        path.push("yt");
                        path.push(
                            &utf8_percent_encode(&yt_id, URI_COMPONENT_ENCODE_SET).to_string(),
                        );
                    }
                    _ => return None,
                };
                Some(url)
            }
            _ => None,
        }
    }

    pub fn youtube_url(&self) -> Option<String> {
        match &self.source {
            StreamSource::YouTube { yt_id } => Some(format!(
                "https://youtube.com/watch?v={}",
                utf8_percent_encode(&yt_id, URI_COMPONENT_ENCODE_SET)
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

    fn archive_query(
        urls: &Vec<ArchiveUrl>,
        file_idx: &Option<u16>,
        file_must_include: &Vec<String>,
        streaming_server_url: &Url,
    ) -> String {
        let payload = serde_json::to_string(&ArchiveStreamBody {
            urls: urls
                .into_iter()
                .filter_map(|archive_url| {
                    Self::ftp_url_handler(streaming_server_url, &archive_url.url).map(|url| {
                        ArchiveUrl {
                            url,
                            bytes: archive_url.bytes,
                        }
                    })
                })
                .collect_vec(),
            options: ArchiveStreamOptions {
                file_idx: *file_idx,
                file_must_include: file_must_include.to_owned(),
            },
        })
        .unwrap();

        let mut query = form_urlencoded::Serializer::new(String::new());
        query.append_pair("lz", &lz_str::compress_to_encoded_uri_component(&payload));

        query.finish()
    }

    /// # Examples
    /// ```
    /// use stremio_core::types::resource::{Stream, StreamSource};
    ///
    /// assert_eq!("file.rar".to_string(), Stream::ftp_filename(&"ftp://example.com/file.rar".parse().unwrap()).unwrap());
    /// assert_eq!("0x00000000000000000000".to_string(), Stream::ftp_filename(&"ftp://example.com/0x00000000000000000000".parse().unwrap()).unwrap());
    /// ```
    pub fn ftp_filename(url: &Url) -> Option<String> {
        url.path_segments()
            .and_then(|segments| segments.last())
            .map(|s| s.to_string())
    }

    /// Converts an `ftp://` or `ftps://` url to a proxied streaming server url
    ///
    /// # Returns
    ///
    /// Err(EnvError::Other) - If streaming server is not available
    /// Err(EnvError::Other) - If filename cannot be extracted from the url, either `/file_name.ext`
    /// or `/0x0adf0120` string path with no extension are supported
    /// Ok(Url) - if stream is converted or left unchanged (non-ftp url)
    fn ftp_url_handler(streaming_server_url: &Url, url: &Url) -> Option<Url> {
        match url.scheme() {
            "ftp" | "ftps" => {
                let filename = Self::ftp_filename(url)?;

                let payload = FtpStreamBody {
                    ftp_url: url.to_owned(),
                };
                let lz_param = serde_json::to_string(&payload).unwrap();

                let mut query = form_urlencoded::Serializer::new(String::new());
                query.append_pair("lz", &lz_str::compress_to_encoded_uri_component(&lz_param));

                let mut url = streaming_server_url.to_owned();
                url.set_path(&format!(
                    "ftp/{filename}/{query}",
                    query = query.finish().as_str(),
                ));

                Some(url)
            }
            _ => None,
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
/// use stremio_core::types::resource::{ArchiveUrl, StreamSource};
///
/// let streams_json = serde_json::json!([
/// {
///     "rarUrls": [["https://example-source.com/file.rar", 10000], ["https://example-source2.com/file2.rar", null ]],
///     // ...Stream
/// },
/// {
///     "rarUrls": [["https://example-source3.com/file.rar"], ["https://example-source4.com/file2.rar"]],
///     "fileIdx": 1,
///     "fileMustInclude": ["includeFile1"],
///     // ...Stream
/// },
/// {
///     "rarUrls": [["https://example-source5.com/file.rar"], ["https://example-source6.com/file2.rar"]],
///     "fileMustInclude": ["includeFile2"],
///     // ...Stream
/// },
/// {
///     "rarUrls": [["https://example-source7.com/file.rar"], ["https://example-source8.com/file2.rar"]],
///     "fileIdx": 2,
///     // ...Stream
/// }
/// ]);
///
/// let expected = vec![
///     StreamSource::Rar {
///         urls: vec![ArchiveUrl { url: "https://example-source.com/file.rar".parse().unwrap(), bytes: Some(10_000) }, ArchiveUrl {url: "https://example-source2.com/file2.rar".parse().unwrap(), bytes: None }],
///         file_idx: None,
///         file_must_include: vec![],
///     },
///     StreamSource::Rar {
///         urls: vec![ArchiveUrl { url: "https://example-source3.com/file.rar".parse().unwrap(), bytes: None }, ArchiveUrl {url: "https://example-source4.com/file2.rar".parse().unwrap(), bytes: None }],
///         file_idx: Some(1),
///         file_must_include: vec!["includeFile1".into()]
///     },
///     StreamSource::Rar {
///         urls: vec![ArchiveUrl { url: "https://example-source5.com/file.rar".parse().unwrap(), bytes: None }, ArchiveUrl {url: "https://example-source6.com/file2.rar".parse().unwrap(), bytes: None }],
///         file_idx: None,
///         file_must_include: vec!["includeFile2".into()]
///     },
///     StreamSource::Rar {
///         urls: vec![
///             ArchiveUrl { url: "https://example-source7.com/file.rar".parse().unwrap(), bytes: None }, ArchiveUrl {url: "https://example-source8.com/file2.rar".parse().unwrap(), bytes: None }
///         ],
///         file_idx: Some(2),
///         file_must_include: vec![],
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
/// use stremio_core::types::resource::{ArchiveUrl, StreamSource};
///
/// let streams_json = serde_json::json!([
/// {
///     "zipUrls": [["https://example-source.com/file.rar", 20000], ["https://example-source2.com/file2.rar"]],
///     // ...Stream
/// },
/// {
///     "zipUrls": [["https://example-source3.com/file.rar"], ["https://example-source4.com/file2.rar"]],
///     "fileIdx": 1,
///     "fileMustInclude": ["includeFile1"],
///     // ...Stream
/// },
/// {
///     "zipUrls": [["https://example-source5.com/file.rar"], ["https://example-source6.com/file2.rar"]],
///     "fileMustInclude": ["includeFile2"],
///     // ...Stream
/// },
/// {
///     "zipUrls": [["https://example-source7.com/file.rar"], ["https://example-source8.com/file2.rar"]],
///     "fileIdx": 2,
///     // ...Stream
/// }
/// ]);
///
/// let expected = vec![
///     StreamSource::Zip {
///         urls: vec![ArchiveUrl {url: "https://example-source.com/file.rar".parse().unwrap(), bytes: Some(20_000) }, ArchiveUrl {url: "https://example-source2.com/file2.rar".parse().unwrap(), bytes: None}],
///         file_idx: None,
///         file_must_include: vec![],
///     },
///     StreamSource::Zip {
///         urls: vec![ArchiveUrl {url: "https://example-source3.com/file.rar".parse().unwrap(), bytes: None}, ArchiveUrl {url: "https://example-source4.com/file2.rar".parse().unwrap(), bytes: None}],
///         file_idx: Some(1),
///         file_must_include: vec!["includeFile1".into()],
///     },
///     StreamSource::Zip {
///         urls: vec![ArchiveUrl {url: "https://example-source5.com/file.rar".parse().unwrap(), bytes: None}, ArchiveUrl {url: "https://example-source6.com/file2.rar".parse().unwrap(), bytes: None}],
///         file_idx: None,
///         file_must_include: vec!["includeFile2".into()],
///     },
///     StreamSource::Zip {
///         urls: vec![ArchiveUrl {url: "https://example-source7.com/file.rar".parse().unwrap(), bytes: None}, ArchiveUrl {url: "https://example-source8.com/file2.rar".parse().unwrap(), bytes: None}],
///         file_idx: Some(2),
///         file_must_include: vec![],
///     },
/// ];
///
/// let streams: Vec<StreamSource> = serde_json::from_value(streams_json).expect("Deserialize all StreamSources");
///
/// pretty_assertions::assert_eq!(streams, expected);
/// ```
#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(derivative::Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(untagged, expecting = "Valid StreamSource")]
pub enum StreamSource {
    Url {
        url: Url,
    },
    #[cfg_attr(test, derivative(Default))]
    #[serde(rename_all = "camelCase")]
    YouTube {
        yt_id: String,
    },
    /// Rar archive source
    #[serde(rename_all = "camelCase")]
    Rar {
        #[serde(rename = "rarUrls")]
        urls: Vec<ArchiveUrl>,
        #[serde(default)]
        file_idx: Option<u16>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[serde_as(deserialize_as = "DefaultOnNull")]
        file_must_include: Vec<String>,
    },
    /// Zip archive source
    #[serde(rename_all = "camelCase")]
    Zip {
        #[serde(rename = "zipUrls")]
        urls: Vec<ArchiveUrl>,
        #[serde(default)]
        file_idx: Option<u16>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[serde_as(deserialize_as = "DefaultOnNull")]
        file_must_include: Vec<String>,
    },
    /// 7zip archive source
    #[serde(rename_all = "camelCase")]
    Zip7 {
        #[serde(rename = "7zipUrls")]
        urls: Vec<ArchiveUrl>,
        #[serde(default)]
        file_idx: Option<u16>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[serde_as(deserialize_as = "DefaultOnNull")]
        file_must_include: Vec<String>,
    },
    /// Tgz archive source
    #[serde(rename_all = "camelCase")]
    Tgz {
        #[serde(rename = "tgzUrls")]
        urls: Vec<ArchiveUrl>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_idx: Option<u16>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[serde_as(deserialize_as = "DefaultOnNull")]
        file_must_include: Vec<String>,
    },
    /// Tar archive source
    #[serde(rename_all = "camelCase")]
    Tar {
        #[serde(rename = "tarUrls")]
        urls: Vec<ArchiveUrl>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_idx: Option<u16>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[serde_as(deserialize_as = "DefaultOnNull")]
        file_must_include: Vec<String>,
    },
    /// Nzb sourced
    #[serde(rename_all = "camelCase")]
    Nzb {
        nzb_url: Url,
        #[serde(default)]
        servers: Vec<Url>,
    },
    #[serde(rename_all = "camelCase")]
    Torrent {
        #[serde(with = "SerHex::<Strict>")]
        info_hash: [u8; 20],
        #[serde(default)]
        file_idx: Option<u16>,
        #[serde_as(deserialize_as = "DefaultOnNull")]
        #[serde(default, alias = "sources")]
        announce: Vec<String>,
        #[serde_as(deserialize_as = "DefaultOnNull")]
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        file_must_include: Vec<String>,
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

/// ```
/// use stremio_core::types::resource::ArchiveUrl;
///
/// let expected = serde_json::json!([
///     ["http://example.com/file0.rar"],
///     ["http://example.com/file1.rar", 123]
/// ]);
/// let archive_urls = vec![ArchiveUrl { url: "http://example.com/file0.rar".parse().unwrap(), bytes: None }, ArchiveUrl { url: "http://example.com/file1.rar".parse().unwrap(), bytes: Some(123) }];
///
/// let ser_stream_source = serde_json::to_value(&archive_urls).expect("Should serialize");
/// assert_eq!(ser_stream_source, expected);
/// println!("{:?}", ser_stream_source);
/// let stream_source = serde_json::from_value::<Vec<ArchiveUrl>>(expected).expect("Should deserialize");
/// assert_eq!(archive_urls, stream_source);
/// ```
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(from = "ArchiveUrlShort", into = "ArchiveUrlShort")]
pub struct ArchiveUrl {
    pub url: Url,
    /// File size (if known) in Bytes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
}

impl fmt::Debug for ArchiveUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchiveUrl")
            .field("url", &self.url.as_str())
            .field("bytes", &self.bytes)
            .finish()
    }
}

impl From<ArchiveUrlShort> for ArchiveUrl {
    fn from(value: ArchiveUrlShort) -> Self {
        Self {
            url: value.0,
            bytes: value.1,
        }
    }
}
impl From<ArchiveUrl> for ArchiveUrlShort {
    fn from(value: ArchiveUrl) -> Self {
        Self(value.url, value.bytes)
    }
}

// TODO:
/// ```
/// use stremio_core::types::resource::ArchiveUrlShort;
///
/// let stream_source = serde_json::from_value::<Vec<ArchiveUrlShort>>(serde_json::json!([
///     ["https://example.com"],
///     ["https://example.com", 123]
/// ]))
/// .expect("Should deserialize");
/// ```
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchiveUrlShort(
    Url,
    #[serde(default, skip_serializing_if = "Option::is_none")] Option<u64>,
);

impl fmt::Debug for ArchiveUrlShort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchiveUrl")
            .field("url", &self.0.as_str())
            .field("bytes", &self.1)
            .finish()
    }
}

type ExternalStreamSource = (Option<Url>, Option<Url>, Option<String>, Option<String>);

pub(crate) fn deserialize_stream_source_external<'de, D>(
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

/// See <https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/responses/stream.md#additional-properties-to-provide-information--behaviour-flags> for documentation
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_size: Option<u64>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

fn is_default_value<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_lz_string_decompress() {
        let url = "http://127.0.0.1:11470/nzb/create?lz=N4IgdgXgRgqgTgGxALhACwC4YA4GdkD0BAJnAK5gDWApmLgmQOYB0AxgPYC2Bj1GkUAgEMAjAA4RAdlYBOSQFYx1GQBYAZqwDMa6cXmyha6spmbRalWM0A2KBOYCAZAEsAvFIAM16x8dxX8tZqHjLUrFBCUAZCmqyRYqxiPvIATB4qItRQ1EIgADQguNRwAG7FuCgA2uBgOPhEUACekpweYNjU1sgAQswpMABqAPpiAEoeUACk8gCiAI4AAmDUAO64zGRFyxhFpcVsXMgqKpoEKvk1dYQEAFZC5BHIMAAilCsDalBqC0WsZHDUByrdY5XCNZZrA6cI4nM4XMC1PDXCjYSQpTSSFQeESaE7IZ4qFZiGSTdEARRUAC1GAANMQAYVJ3Umkm6AA8ABSklTTGZiFIAWlJmmmzxuC2oZA2Wz42Dgzk4gI40OOp3OBQRVyIrDiKhS1E0gREevkeigyEmmgAggB5STEXnsSmW%2BnWGRWoRLYHMVbUShCBwIZCBNUgAC6AF8gA".parse::<url::Url>().unwrap();
        let lz_str = url
            .query_pairs()
            .find_map(|(key, value)| {
                if key == "lz" {
                    Some(value.to_string())
                } else {
                    None
                }
            })
            .unwrap();
        let decomp =
            lz_str::decompress_from_encoded_uri_component(&lz_str).expect("Should decompress");

        let decomp_string =
            String::from_utf16(&decomp).expect("Decompressed data is not valid UTF-16");

        println!(
            "Compressed string length: {}\nServer Url total Length: {}\n\t{decomp_string}",
            decomp_string.len(),
            url.as_str().len(),
        );
    }
}
