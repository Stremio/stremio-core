use core::fmt;
use std::{collections::HashMap, io::Write};

use tracing::trace;

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
        streams::StreamSourceTrait,
        torrent::InfoHash,
    },
};
use crate::{runtime::EnvError, types::streams::ConvertedStreamSource};

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
pub struct Stream<S: StreamSourceTrait = StreamSource> {
    // pub struct Stream {
    #[serde(flatten)]
    pub source: S,
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

impl<S> Stream<S>
where
    S: StreamSourceTrait + Serialize,
{
    pub fn encode(&self) -> Result<String, anyhow::Error> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::none());
        let stream = serde_json::to_string(&self)?;
        encoder.write_all(stream.as_bytes())?;
        let stream = encoder.finish()?;
        let stream = BASE64.encode(stream);
        Ok(stream)
    }
}

impl<S> Stream<S>
where
    S: StreamSourceTrait + Serialize + serde::de::DeserializeOwned,
{
    pub fn decode(stream: &str) -> Result<Self, anyhow::Error> {
        let stream = BASE64.decode(stream)?;
        let mut writer = Vec::new();
        let mut decoder = ZlibDecoder::new(writer);
        decoder.write_all(&stream)?;
        writer = decoder.finish()?;
        let stream = String::from_utf8(writer)?;
        let stream = serde_json::from_str(&stream)?;
        Ok(stream)
    }
}

impl Stream {
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

    pub fn to_converted(&self, converted: ConvertedStreamSource) -> Stream<ConvertedStreamSource> {
        Stream {
            source: converted,
            name: self.name.clone(),
            description: self.description.clone(),
            thumbnail: self.thumbnail.clone(),
            subtitles: self.subtitles.clone(),
            behavior_hints: self.behavior_hints.clone(),
        }
    }

    /// # Examples
    /// ```
    /// use stremio_core::types::resource::{Stream, StreamSource};
    ///
    /// assert_eq!("file.rar".to_string(), Stream::ftp_filename(&"ftp://example.com/file.rar".parse().unwrap()).unwrap());
    /// assert_eq!("0x00000000000000000000".to_string(), Stream::ftp_filename(&"ftp://example.com/0x00000000000000000000".parse().unwrap()).unwrap());
    /// ```
    pub fn ftp_filename(url: &Url) -> Result<String, EnvError> {
        url.path_segments()
            .and_then(|segments| segments.last())
            .map(|s| s.to_string())
            .ok_or(EnvError::Other(
                "Ftp(s) filepath is missing in the url".into(),
            ))
    }

    /// Converts an `ftp://` or `ftps://` url to a proxied streaming server url
    ///
    /// # Returns
    ///
    /// Err(EnvError::Other) - If streaming server is not available
    /// Err(EnvError::Other) - If filename cannot be extracted from the url, either `/file_name.ext`
    /// or `/0x0adf0120` string path with no extension are supported
    /// Ok(Url) - if stream is converted or left unchanged (non-ftp url)
    fn ftp_url_handler(streaming_server_url: Option<&Url>, url: Url) -> Result<Url, EnvError> {
        match (streaming_server_url, url.scheme()) {
            (Some(streaming_server_url), "ftp") | (Some(streaming_server_url), "ftps") => {
                let filename = Self::ftp_filename(&url)?;

                let mut stream_url = streaming_server_url
                    .join("ftp/")
                    .map_err(|err| EnvError::Other(err.to_string()))?
                    .join(&filename)
                    .map_err(|err| EnvError::Other(err.to_string()))?;

                let payload = FtpStreamBody { ftp_url: url };

                let stream_data = serde_json::to_string(&payload)?;
                stream_url.query_pairs_mut().append_pair(
                    "lz",
                    &lz_str::compress_to_encoded_uri_component(&stream_data),
                );

                trace!(%stream_url, json_payload=?payload, "Ftp(s) Streaming Server Request");

                Ok(stream_url)
            }
            (None, "ftp") | (None, "ftps") => Err(EnvError::Other(
                "Can't play Ftp(s) because streaming server is not running".into(),
            )),
            _ => Ok(url),
        }
    }

    /// Updates a `StreamSource` if it's Rar, Zip, 7zip, Tar, Tgz and Nzb
    /// and creates a [`ConvertedStreamSource::Url`]
    pub fn convert(
        &self,
        streaming_server_url: Option<&Url>,
    ) -> Result<Stream<ConvertedStreamSource>, EnvError> {
        match (streaming_server_url, self.source.to_owned()) {
            (
                Some(streaming_server_url),
                StreamSource::Rar {
                    urls,
                    file_idx,
                    file_must_include,
                },
            ) => {
                if urls.is_empty() {
                    return Err(EnvError::Other("No Rar URLs provided".into()));
                }

                let mut stream_url = streaming_server_url
                    .join("rar/create")
                    .map_err(|err| EnvError::Other(err.to_string()))?;

                let payload = ArchiveStreamBody {
                    urls: urls
                        .into_iter()
                        .map(|archive_url| {
                            Self::ftp_url_handler(Some(streaming_server_url), archive_url.url).map(
                                |url| ArchiveUrl {
                                    url,
                                    bytes: archive_url.bytes,
                                },
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .expect("Streaming server availability is already checked"),
                    options: ArchiveStreamOptions {
                        file_idx,
                        file_must_include,
                    },
                };

                let stream_data = serde_json::to_string(&payload)?;
                stream_url.query_pairs_mut().append_pair(
                    "lz",
                    &lz_str::compress_to_encoded_uri_component(&stream_data),
                );
                trace!(%stream_url, json_payload=?payload, "Rar Streaming Server Request");

                Ok(self.to_converted(ConvertedStreamSource::Url { url: stream_url }))
            }
            (
                Some(streaming_server_url),
                StreamSource::Zip {
                    urls,
                    file_idx,
                    file_must_include,
                },
            ) => {
                if urls.is_empty() {
                    return Err(EnvError::Other("No Zip URLs provided".into()));
                }

                let mut stream_url = streaming_server_url
                    .clone()
                    .join(&format!("zip/create"))
                    .expect("Url should always be valid");

                let payload = ArchiveStreamBody {
                    urls: urls
                        .into_iter()
                        .map(|archive_url| {
                            Self::ftp_url_handler(Some(streaming_server_url), archive_url.url).map(
                                |url| ArchiveUrl {
                                    url,
                                    bytes: archive_url.bytes,
                                },
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .expect("Streaming server availability is already checked"),
                    options: ArchiveStreamOptions {
                        file_idx,
                        file_must_include,
                    },
                };

                let stream_data = serde_json::to_string(&payload)?;
                stream_url.query_pairs_mut().append_pair(
                    "lz",
                    &lz_str::compress_to_encoded_uri_component(&stream_data),
                );

                trace!(stream_url=%stream_url, json_payload=?payload, "Zip Streaming Server Request");

                Ok(self.to_converted(ConvertedStreamSource::Url { url: stream_url }))
            }
            (
                Some(streaming_server_url),
                StreamSource::Zip7 {
                    urls,
                    file_idx,
                    file_must_include,
                },
            ) => {
                if urls.is_empty() {
                    return Err(EnvError::Other("No 7zip URLs provided".into()));
                }

                let mut stream_url = streaming_server_url
                    .clone()
                    .join(&format!("7zip/create"))
                    .expect("Url should always be valid");
                let payload = ArchiveStreamBody {
                    urls: urls
                        .into_iter()
                        .map(|archive_url| {
                            Self::ftp_url_handler(Some(streaming_server_url), archive_url.url).map(
                                |url| ArchiveUrl {
                                    url,
                                    bytes: archive_url.bytes,
                                },
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .expect("Streaming server availability is already checked"),
                    options: ArchiveStreamOptions {
                        file_idx,
                        file_must_include,
                    },
                };

                let stream_data = serde_json::to_string(&payload)?;
                stream_url.query_pairs_mut().append_pair(
                    "lz",
                    &lz_str::compress_to_encoded_uri_component(&stream_data),
                );
                trace!(%stream_url, json_payload=?payload, "7Zip Streaming Server Request");

                Ok(self.to_converted(ConvertedStreamSource::Url { url: stream_url }))
            }
            (
                Some(streaming_server_url),
                StreamSource::Tgz {
                    urls,
                    file_idx,
                    file_must_include,
                },
            ) => {
                if urls.is_empty() {
                    return Err(EnvError::Other("No tgz URLs provided".into()));
                }

                let mut stream_url = streaming_server_url
                    .clone()
                    .join(&format!("tgz/create"))
                    .expect("Url should always be valid");

                let payload = ArchiveStreamBody {
                    urls: urls
                        .into_iter()
                        .map(|archive_url| {
                            Self::ftp_url_handler(Some(streaming_server_url), archive_url.url).map(
                                |url| ArchiveUrl {
                                    url,
                                    bytes: archive_url.bytes,
                                },
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .expect("Streaming server availability is already checked"),
                    options: ArchiveStreamOptions {
                        file_idx,
                        file_must_include,
                    },
                };

                let stream_data = serde_json::to_string(&payload)?;
                stream_url.query_pairs_mut().append_pair(
                    "lz",
                    &lz_str::compress_to_encoded_uri_component(&stream_data),
                );
                trace!(%stream_url, json_payload=?payload, "Tgz Streaming Server Request");

                Ok(self.to_converted(ConvertedStreamSource::Url { url: stream_url }))
            }
            (
                Some(streaming_server_url),
                StreamSource::Tar {
                    urls,
                    file_idx,
                    file_must_include,
                },
            ) => {
                if urls.is_empty() {
                    return Err(EnvError::Other("No tar URLs provided".into()));
                }

                let payload = ArchiveStreamBody {
                    urls: urls
                        .into_iter()
                        .map(|archive_url| {
                            Self::ftp_url_handler(Some(streaming_server_url), archive_url.url).map(
                                |url| ArchiveUrl {
                                    url,
                                    bytes: archive_url.bytes,
                                },
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .expect("Streaming server availability is already checked"),
                    options: ArchiveStreamOptions {
                        file_idx,
                        file_must_include,
                    },
                };

                let mut stream_url = streaming_server_url
                    .clone()
                    .join(&format!("tar/create"))
                    .expect("Url should always be valid");

                let stream_data = serde_json::to_string(&payload)?;
                stream_url.query_pairs_mut().append_pair(
                    "lz",
                    &lz_str::compress_to_encoded_uri_component(&stream_data),
                );
                trace!(%stream_url, json_payload=?payload, "Tar Streaming Server Request");

                Ok(self.to_converted(ConvertedStreamSource::Url { url: stream_url }))
            }
            (Some(streaming_server_url), StreamSource::Nzb { nzb_url, servers }) => {
                if servers.is_empty() {
                    return Err(EnvError::Other("No nzb server URLs provided".into()));
                }

                let servers = servers
                    .into_iter()
                    .map(|server_url| Self::ftp_url_handler(Some(streaming_server_url), server_url))
                    .collect::<Result<Vec<_>, _>>()
                    .expect("Streaming server availability is already checked");

                let mut stream_url = streaming_server_url
                    .clone()
                    .join(&format!("nzb/create"))
                    .expect("Url should always be valid");

                let payload = StreamSource::Nzb {
                    nzb_url: Self::ftp_url_handler(Some(streaming_server_url), nzb_url)
                        .expect("Streaming server availability is already checked"),
                    servers,
                };

                let stream_data = serde_json::to_string(&payload)?;
                stream_url.query_pairs_mut().append_pair(
                    "lz",
                    &lz_str::compress_to_encoded_uri_component(&stream_data),
                );
                trace!(%stream_url, json_payload=?payload, "Nzb Streaming Server Request");

                Ok(self.to_converted(ConvertedStreamSource::Url { url: stream_url }))
            }
            (
                None,
                StreamSource::Rar { .. }
                | StreamSource::Zip { .. }
                | StreamSource::Zip7 { .. }
                | StreamSource::Tgz { .. }
                | StreamSource::Tar { .. }
                | StreamSource::Nzb { .. },
            ) => Err(EnvError::Other(
                "Can't play Rar/Zip/Zip7/Tar/Tgz/Nzb because the streaming server is not running"
                    .into(),
            )),
            // no further changes are needed for now
            // we still need to create torrents, etc. in stremio-video
            // as it's not part of the current scope
            // This keeps the `magnet:` urls working until we get the the streaming url
            (streaming_server_url, StreamSource::Url { url }) if url.scheme() != "magnet" => {
                let url = Self::ftp_url_handler(streaming_server_url, url)?;

                // If proxy headers are set and streaming server is available, build the proxied streaming url from streaming server url
                // Otherwise return the url
                let url = match (&self.behavior_hints.proxy_headers, streaming_server_url) {
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
                        streaming_url
                    }
                    _ => url.to_owned(),
                };

                Ok(self.to_converted(ConvertedStreamSource::Url { url }))
            }
            // Magnet URL stream source handling
            // we keep the magnet url and return None for Steaming url later on
            (_streaming_server_url, StreamSource::Url { url: streaming_url }) => {
                Ok(self.to_converted(ConvertedStreamSource::Url { url: streaming_url }))
            }
            (Some(streaming_server_url), StreamSource::YouTube { yt_id }) => {
                Ok(self.to_converted(ConvertedStreamSource::YouTube {
                    url: {
                        let mut url = streaming_server_url.to_owned();
                        {
                            let mut path = url
                                .path_segments_mut()
                                .expect("Streaming server should always be base");
                            path.push("yt");
                            path.push(
                                &utf8_percent_encode(&yt_id, URI_COMPONENT_ENCODE_SET).to_string(),
                            );
                        }

                        url
                    },
                    yt_id,
                }))
            }
            (None, StreamSource::YouTube { .. }) => Err(EnvError::Other(
                "Can't play Youtube videos because streaming server is not running".into(),
            )),
            // Torrent stream source handling
            (
                Some(streaming_server_url),
                StreamSource::Torrent {
                    info_hash,
                    file_idx,
                    announce,
                    file_must_include,
                },
            ) => {
                let streaming_url = {
                    let mut url = streaming_server_url.to_owned();

                    {
                        let mut path = url.path_segments_mut().expect("Should always be base url");
                        path.extend([
                            &hex::encode(info_hash),
                            // When fileIndex is not provided use -1, which will tell the
                            // streaming server to choose the file with the largest size from the torrent
                            &file_idx.map_or_else(|| "-1".to_string(), |idx| idx.to_string()),
                        ]);
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
                    url
                };

                Ok(self.to_converted(ConvertedStreamSource::Torrent {
                    // this needs to change once conversion of torrents is also moved to core
                    url: streaming_url,
                    info_hash: InfoHash::new(info_hash),
                    file_idx,
                    announce,
                    file_must_include,
                }))
            }
            (None, StreamSource::Torrent { .. }) => Err(EnvError::Other(
                "Can't play Torrents because streaming server is not running".into(),
            )),
            (_, StreamSource::PlayerFrame { player_frame_url }) => {
                Ok(self.to_converted(ConvertedStreamSource::PlayerFrame { player_frame_url }))
            }
            (
                _,
                StreamSource::External {
                    external_url,
                    android_tv_url,
                    tizen_url,
                    webos_url,
                },
            ) => Ok(self.to_converted(ConvertedStreamSource::External {
                external_url,
                android_tv_url,
                tizen_url,
                webos_url,
            })),
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

#[derive(Clone, derivative::Derivative, Serialize, PartialEq, Eq)]
pub struct StreamUrls {
    #[serde(default)]
    pub magnet_url: Option<Url>,
    #[serde(default)]
    pub download_url: Option<Url>,
    #[serde(default)]
    pub streaming_url: Option<Url>,
    #[serde(default)]
    pub m3u_data_uri: Option<String>,
    /// The Stream for which the Urls were generated
    /// This is very important to have since the stream can change in a model
    /// and allows us it check if the Urls are generated for the same Stream.
    pub stream: Stream<ConvertedStreamSource>,
}

impl fmt::Debug for StreamUrls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamUrls")
            .field(
                "magnet_url",
                &self.magnet_url.as_ref().map(ToString::to_string),
            )
            .field(
                "download_url",
                &self.download_url.as_ref().map(ToString::to_string),
            )
            .field(
                "streaming_url",
                &self.streaming_url.as_ref().map(ToString::to_string),
            )
            .field("m3u_data_uri", &self.m3u_data_uri)
            .field("stream", &self.stream)
            .finish()
    }
}

impl StreamUrls {
    /// For a stream with an already converted source we can directly use the URL
    pub fn new(
        converted: Stream<ConvertedStreamSource>,
        streaming_server_url: Option<&Url>,
    ) -> Self {
        let streaming_url = get_streaming_url(&converted);
        let download_url = get_download_url(&converted, streaming_server_url);
        let magnet_url = get_magnet_url(&converted);

        let m3u_data_uri = streaming_url.as_ref().map(|url| get_m3u_data_uri(url));

        Self {
            magnet_url,
            download_url,
            streaming_url,
            m3u_data_uri,
            stream: converted.to_owned(),
        }
    }
}

fn get_magnet_url(converted: &Stream<ConvertedStreamSource>) -> Option<Url> {
    match &converted.source {
        ConvertedStreamSource::Url { url } if url.scheme() == "magnet" => Magnet::new(url.as_str())
            .as_ref()
            .map(ToString::to_string)
            .ok()
            .and_then(|url_string| url_string.parse().ok()),
        //Current Stream::download_url gets the magnet link and returns it as a download_url
        ConvertedStreamSource::Torrent {
            info_hash,
            announce,
            ..
        } => {
            let torrent_magnet = Magnet {
                dn: converted.name.to_owned(),
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
            };

            torrent_magnet.to_string().parse().ok()
        }
        _ => None,
    }
}

fn get_m3u_data_uri(streaming_url: &Url) -> String {
    format!(
        "data:application/octet-stream;charset=utf-8;base64,{}",
        BASE64.encode(format!("#EXTM3U\n#EXTINF:0\n{streaming_url}"))
    )
}

fn get_download_url(
    converted: &Stream<ConvertedStreamSource>,
    streaming_server_url: Option<&Url>,
) -> Option<Url> {
    match &converted.source {
        ConvertedStreamSource::Url { url } if url.scheme() == "magnet" => Magnet::new(url.as_str())
            .as_ref()
            .map(ToString::to_string)
            .ok()
            .and_then(|url_string| url_string.parse().ok()),
        ConvertedStreamSource::Url { url } => Some(url.to_owned()),
        ConvertedStreamSource::Torrent { url, .. } => {
            // we just need to know that the server is running
            streaming_server_url.as_ref().map(|_| {
                let mut torrent_stream_url = url.clone();
                {
                    let mut query_pairs = torrent_stream_url.query_pairs_mut();
                    query_pairs
                        // clear any existing query parameters!
                        .clear()
                        .append_pair("external", "1")
                        .append_pair("download", "1");
                }
                torrent_stream_url
            })
        }
        // generate the Youtube video URL instead of providing the streaming url
        ConvertedStreamSource::YouTube { yt_id, .. } => Some(
            format!(
                "https://youtube.com/watch?v={}",
                utf8_percent_encode(yt_id.as_str(), URI_COMPONENT_ENCODE_SET)
            )
            .parse()
            .expect("Should always be a valid URL"),
        ),
        ConvertedStreamSource::External { external_url, .. } => external_url.as_ref().cloned(),
        ConvertedStreamSource::PlayerFrame { player_frame_url } => {
            Some(player_frame_url.to_owned())
        }
    }
}

fn get_streaming_url(converted: &Stream<ConvertedStreamSource>) -> Option<Url> {
    match &converted.source {
        ConvertedStreamSource::Url { url } if url.scheme() == "magnet" => None,
        ConvertedStreamSource::Url { url: streaming_url } => Some(streaming_url.to_owned()),
        ConvertedStreamSource::Torrent {
            url: streaming_url, ..
        } => Some(streaming_url.to_owned()),
        ConvertedStreamSource::YouTube {
            url: streaming_url, ..
        } => Some(streaming_url.clone()),
        _ => None,
    }
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
