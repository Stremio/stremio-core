use core::fmt;

use http::Request;
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FtpStreamBody {
    pub ftp_url: Url,
}

impl fmt::Debug for FtpStreamBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FtpStreamBody")
            .field("ftp_url", &self.ftp_url.as_str())
            .finish()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_idx: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_must_include: Vec<String>,
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

#[derive(Debug, Clone)]
pub struct CreateMagnetRequest {
    pub server_url: Url,
    pub info_hash: InfoHash,
    pub announce: Vec<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMagnetBody {
    pub stream: CreateMagnetTorrent,
    pub peer_search: Option<PeerSearch>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMagnetTorrent {
    pub info_hash: InfoHash,
}

impl From<CreateMagnetRequest> for Request<CreateMagnetBody> {
    fn from(val: CreateMagnetRequest) -> Self {
        let info_hash = val.info_hash;

        let body = CreateMagnetBody {
            stream: CreateMagnetTorrent {
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
