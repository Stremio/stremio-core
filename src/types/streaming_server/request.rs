use std::iter;

use http::{header::CONTENT_TYPE, Request};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::types::{streaming_server::PeerSearch, torrent::InfoHash};

use super::CreatedTorrent;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsRequest {
    pub info_hash: String,
    pub file_idx: u16,
}

#[cfg(feature = "experimental")]
pub struct ArchiveStreamRequest {
    /// The `rar/create` or `zip/create` key returned in the response
    pub response_key: String,
    pub options: ArchiveStreamOptions,
}

#[cfg(feature = "experimental")]
impl ArchiveStreamRequest {
    pub fn to_query_pairs(self) -> Vec<(String, String)> {
        let options = serde_json::to_value(&self.options).expect("should serialize");
        let options_object = options.as_object().expect("Should be an object");

        vec![
            (
                "key".into(),
                // append the length of the options
                // keep in mind that `None` options should always be treated as not-set
                // i.e. should not be serialized
                format!(
                    "{key}{length}",
                    key = self.response_key,
                    length = options_object.len()
                ),
            ),
            ("o".into(), options.to_string()),
        ]
    }
}

/// Server's `rar/stream` and `zip/stream` options of the query.
///
/// Format: `rar/stream?key={create_key}{options length}&o={options_json_string}`
///
/// Where all parameters are url encoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "experimental")]
pub struct ArchiveStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_idx: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_must_include: Vec<String>,
}

pub struct CreateTorrentBlobRequest {
    pub server_url: Url,
    pub torrent: Vec<u8>,
}

impl From<CreateTorrentBlobRequest> for Request<CreateTorrentBlobBody> {
    fn from(val: CreateTorrentBlobRequest) -> Self {
        let endpoint = val.server_url.join("/create").expect("url builder failed");

        Request::post(endpoint.as_str())
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(CreateTorrentBlobBody {
                blob: hex::encode(val.torrent),
            })
            .expect("request builder failed")
    }
}
#[derive(Serialize)]
pub struct CreateTorrentBlobBody {
    pub blob: String,
}

/// # Examples
///
/// Example which creates a request url with body for the server:
/// `http://127.0.0.1:11470/df389295484b3059a4726dc6d8a57f71bb5f4c81/1?tr=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4`
///
/// ```
/// use stremio_core::types::streaming_server::{CreateTorrentRequest, CreatedTorrent};
///
/// let request: http::Request<CreatedTorrent> = CreateTorrentRequest {
///     server_url: "http://127.0.0.1:11470/".parse().unwrap(),
///     sources: vec!["https://example.com/my-awesome-video.mp4".into()],
///     info_hash: "df389295484b3059a4726dc6d8a57f71bb5f4c81".parse().unwrap(),
///     file_idx: 1,
/// }.into();
///
/// assert_eq!("http://127.0.0.1:11470/df389295484b3059a4726dc6d8a57f71bb5f4c81/1?tr=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4", request.uri().to_string());
///  // TODO: assert_eq!(&(), request.body());
/// ```
pub struct CreateTorrentRequest {
    pub server_url: Url,
    pub sources: Vec<String>,
    pub info_hash: InfoHash,
    pub file_idx: u64,
}

impl From<CreateTorrentRequest> for Request<CreatedTorrent> {
    fn from(val: CreateTorrentRequest) -> Self {
        let url = {
            let mut uri = val
                .server_url
                .join(&format!("{}/{}", val.info_hash, val.file_idx))
                .expect("Should always be valid Url");

            {
                let mut x = uri.query_pairs_mut();
                for source in &val.sources {
                    x.append_pair("tr", source);
                }
            }

            uri
        };

        let create_torrent = CreatedTorrent {
            torrent: super::Torrent {
                info_hash: val.info_hash,
            },
            peer_search: PeerSearch::new(40, 200, val.info_hash, val.sources),
            guess_file_idx: None,
        };

        Request::builder()
            .uri(url.as_str())
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .body(create_torrent)
            .expect("Should always be valid Request!")
    }
}

pub struct CreateMagnetRequest {
    pub server_url: Url,
    pub info_hash: InfoHash,
    pub announce: Vec<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMagnetBody {
    pub torrent: CreateMagnetTorrent,
    pub peer_search: Option<PeerSearch>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMagnetTorrent {
    pub info_hash: InfoHash,
}

impl From<CreateMagnetRequest> for Request<CreateMagnetBody> {
    fn from(val: CreateMagnetRequest) -> Self {
        let info_hash = val.info_hash;

        let body = CreateMagnetBody {
            torrent: CreateMagnetTorrent {
                info_hash: val.info_hash.to_owned(),
            },
            peer_search: if !val.announce.is_empty() {
                Some(PeerSearch {
                    sources: iter::once(&format!("dht:{info_hash}"))
                        .chain(val.announce.iter())
                        .cloned()
                        .collect(),
                    min: 40,
                    max: 200,
                })
            } else {
                None
            },
        };

        let info_hash = info_hash.to_owned();
        let endpoint = val
            .server_url
            .join(&format!("{info_hash}/create"))
            .expect("url builder failed");

        Request::post(endpoint.as_str())
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(body)
            .expect("request builder should never fail!")
    }
}

/// # Examples
///
/// Example which creates a request url with no body for the server:
/// `http://127.0.0.1:11470/opensubHash?videoUrl=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4`
///
/// ```
/// use stremio_core::types::streaming_server::OpensubtitlesParamsRequest;
///
/// let request: http::Request<()> = OpensubtitlesParamsRequest {
///     server_url: "http://127.0.0.1:11470/".parse().unwrap(),
///     media_url: "https://example.com/my-awesome-video.mp4".parse().unwrap(),
///     
/// }.into();
///
/// assert_eq!("http://127.0.0.1:11470/opensubHash?videoUrl=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4", request.uri().to_string());
/// assert_eq!(&(), request.body());
/// ```
#[cfg(feature = "experimental")]
pub struct OpensubtitlesParamsRequest {
    pub server_url: Url,
    pub media_url: Url,
}

#[cfg(feature = "experimental")]
impl Into<Request<()>> for OpensubtitlesParamsRequest {
    fn into(self) -> Request<()> {
        // var queryParams = new URLSearchParams([['videoUrl', mediaURL]]);
        // return fetch(url.resolve(streamingServerURL, '/opensubHash?' + queryParams.toString()))
        let url = {
            let mut uri = self
                .server_url
                .join("opensubHash")
                .expect("Should always be valid Url");
            {
                let mut x = uri.query_pairs_mut();
                x.append_pair("videoUrl", self.media_url.as_str());
            }

            uri
        };

        Request::builder()
            .uri(url.as_str())
            .body(())
            .expect("Should always be valid Request!")
    }
}

/// Filename request to the server.
///
/// `{streaming_sever_url}/{info_hash_url_encoded}/{file_idx_url_encoded}/stats.json`
///
///
/// Example: `http://127.0.0.1:11470/6d0cdb871b81477d00f53f78529028994b364877/7/stats.json`
pub struct TorrentStatisticsRequest {
    pub server_url: Url,
    pub request: StatisticsRequest,
}
impl From<TorrentStatisticsRequest> for Request<()> {
    fn from(val: TorrentStatisticsRequest) -> Self {
        let info_hash_encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_key_only(&val.request.info_hash.to_string())
            .finish();
        let file_idx_encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_key_only(&val.request.file_idx.to_string())
            .finish();

        let uri = val
            .server_url
            .join(&format!(
                "{info_hash_encoded}/{file_idx_encoded}/stats.json"
            ))
            .expect("Should always be valid url!");

        Request::get(uri.as_str())
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(())
            .expect("Always valid request!")
    }
}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "experimental")]
    fn test_options_to_serde_json_value_keys_length() {
        use super::ArchiveStreamOptions;
        // 0 keys
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: None,
                file_must_include: vec![],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert!(object.is_empty());
        }

        // only fileIdx
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: Some(1),
                file_must_include: vec![],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert_eq!(1, object.len(), "Only fileIdx is set");
            assert_eq!(object.keys().next().cloned(), Some("fileIdx".to_string()));
        }

        // both keys are set
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: Some(1),
                file_must_include: vec!["fileName".into(), "nameFile".into()],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert_eq!(2, object.len(), "Only fileIdx is set");
        }
    }
}
