use http::{header::CONTENT_TYPE, Request};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::types::{resource::ArchiveUrl, streaming_server::PeerSearch, torrent::InfoHash};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Request used for `rar`, `zip`, `7zip`, tgz & tar creation
/// the only difference is with `nzb` which expects `nzbUrl` & `servers` fields
pub struct ArchiveStreamBody {
    /// The `rar/create`, `zip/create`, `7zip/create`, `tgz/create`, `tar/create` urls
    pub urls: Vec<ArchiveUrl>,
    #[serde(flatten)]
    pub options: ArchiveStreamOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FtpStreamBody {
    pub ftp_url: Url,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_idx: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_must_include: Vec<String>,
}

/// # Examples
///
/// Example which creates a request url with no body for the server:
/// `http://127.0.0.1:11470/opensubHash?videoUrl=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4`
///
/// ```
/// use stremio_core::types::streaming_server::CreateTorrentRequest;
///
/// let request: http::Request<()> = CreateTorrentRequest {
///     server_url: "http://127.0.0.1:11470/".parse().unwrap(),
///     sources: vec!["https://example.com/my-awesome-video.mp4".parse().unwrap()],
///     info_hash: "017d177431e71199c96fbf2d4dee312471560af1".parse().unwrap(),
///     file_idx: 1,
///     
/// }.into();
///
/// assert_eq!("http://127.0.0.1:11470/017d177431e71199c96fbf2d4dee312471560af1/1?tr=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4", request.uri().to_string());
/// assert_eq!(&(), request.body());
/// ```
pub struct CreateTorrentRequest {
    pub server_url: Url,
    pub sources: Vec<Url>,
    pub info_hash: InfoHash,
    pub file_idx: u64,
}

impl From<CreateTorrentRequest> for Request<()> {
    fn from(value: CreateTorrentRequest) -> Self {
        // var query = Array.isArray(sources) && sources.length > 0 ?
        //     '?' + new URLSearchParams(sources.map(function(source) {
        //         return ['tr', source];
        //     }))
        //     :
        //     '';
        // return {
        //     url: url.resolve(streamingServerURL, '/' + encodeURIComponent(infoHash) + '/' + encodeURIComponent(fileIdx)) + query,
        //     infoHash: infoHash,
        //     fileIdx: fileIdx,
        //     sources: sources
        // }
        let url = {
            let mut uri = value
                .server_url
                .join(&format!("{}/{}", value.info_hash, value.file_idx))
                .expect("Should always be valid Url");

            {
                let mut x = uri.query_pairs_mut();
                if !value.sources.is_empty() {
                    for source in value.sources {
                        x.append_pair("tr", source.as_str());
                    }
                }
            }

            uri
        };

        Request::builder()
            .uri(url.as_str())
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .body(())
            .expect("Should always be valid Request!")
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
pub struct OpensubtitlesParamsRequest {
    pub server_url: Url,
    pub media_url: Url,
}

impl From<OpensubtitlesParamsRequest> for Request<()> {
    fn from(value: OpensubtitlesParamsRequest) -> Self {
        let url = {
            let mut uri = value
                .server_url
                .join("opensubHash")
                .expect("Should always be valid Url");
            {
                let mut x = uri.query_pairs_mut();

                x.append_pair("videoUrl", value.media_url.as_str());
            }

            // x.finish();
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
/// `{streaming_sever_url}/{info_hash_url_encoded}/{file_idx_url_encoded/stats.json`
///
///
/// Example: `http://127.0.0.1:11470/6d0cdb871b81477d00f53f78529028994b364877/7/stats.json`
pub struct FileNameRequest {
    pub server_url: Url,
    pub request: StatisticsRequest,
    // pub info_hash: InfoHash,
    // pub file_idx: u64,
}

impl From<FileNameRequest> for Request<()> {
    fn from(val: FileNameRequest) -> Self {
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

        Request::builder()
            .uri(uri.to_string())
            .body(())
            .expect("Always valid request!")
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsRequest {
    pub info_hash: String,
    pub file_idx: u16,
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
                Some(PeerSearch::new(40, 200, info_hash, val.announce))
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
