use http::header;

use crate::types::streaming_server::{
    CreateTorrentBlobRequest, StatisticsRequest, TorrentStatisticsRequest,
};

const INFO_HASH: &str = "120c34efc4b29192d7c504344a39a7b0c2c6da21";

#[test]
fn create_torrent_blob_uses_credentials_and_a_clean_url() {
    let request: http::Request<_> = CreateTorrentBlobRequest {
        server_url: "https://user:pass@host/stremio/".parse().unwrap(),
        torrent: vec![0u8; 1],
    }
    .into();
    assert_eq!(request.uri().to_string(), "https://host/stremio/create");
    assert_eq!(
        request.headers().get(header::AUTHORIZATION),
        Some(&header::HeaderValue::from_static("Basic dXNlcjpwYXNz"))
    );
}

#[test]
fn statistics_uses_credentials_and_a_clean_url() {
    let request: http::Request<()> = TorrentStatisticsRequest {
        server_url: "https://user:pass@host/stremio/".parse().unwrap(),
        request: StatisticsRequest {
            info_hash: INFO_HASH.to_owned(),
            file_idx: None,
        },
    }
    .into();
    assert_eq!(
        request.uri().to_string(),
        format!("https://host/stremio/{INFO_HASH}/stats.json")
    );
    assert_eq!(
        request.headers().get(header::AUTHORIZATION),
        Some(&header::HeaderValue::from_static("Basic dXNlcjpwYXNz"))
    );
}

#[test]
fn requests_without_credentials_have_no_authorization_header() {
    let request: http::Request<()> = TorrentStatisticsRequest {
        server_url: "https://host/stremio/".parse().unwrap(),
        request: StatisticsRequest {
            info_hash: INFO_HASH.to_owned(),
            file_idx: None,
        },
    }
    .into();
    assert!(request.headers().get(header::AUTHORIZATION).is_none());
}
