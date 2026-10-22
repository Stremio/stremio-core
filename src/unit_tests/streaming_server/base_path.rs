use core::str::FromStr;

use http::Request;
use url::Url;

use crate::types::{
    profile::Settings,
    streaming_server::{
        CreateMagnetRequest, CreateTorrentBlobRequest, StatisticsRequest, TorrentStatisticsRequest,
    },
    torrent::InfoHash,
};

const INFO_HASH: &str = "aabbccddeeff00112233445566778899aabbccdd";

// The streaming server is configured with a trailing slash (the settings
// deserializer normalizes it), so every relative `Url::join` below must keep
// the /stremio path prefix.
fn base() -> Url {
    Url::parse("https://host/stremio/").unwrap()
}

#[test]
fn streaming_server_url_is_normalized_to_a_directory() {
    let mut settings = Settings::default();
    settings.streaming_server_url = Url::parse("https://host/stremio").unwrap();
    let parsed: Settings =
        serde_json::from_value(serde_json::to_value(&settings).unwrap()).unwrap();
    assert_eq!(
        parsed.streaming_server_url.as_str(),
        "https://host/stremio/"
    );
}

#[test]
fn create_torrent_blob_endpoint_keeps_base_path() {
    let req: Request<_> = CreateTorrentBlobRequest {
        server_url: base(),
        torrent: vec![0u8; 4],
    }
    .into();
    assert_eq!(req.uri().to_string(), "https://host/stremio/create");
}

#[test]
fn create_magnet_endpoint_keeps_base_path() {
    let req: Request<_> = CreateMagnetRequest {
        server_url: base(),
        info_hash: InfoHash::from_str(INFO_HASH).unwrap(),
        announce: vec![],
    }
    .into();
    assert_eq!(
        req.uri().to_string(),
        format!("https://host/stremio/{INFO_HASH}/create")
    );
}

#[test]
fn statistics_endpoint_keeps_base_path() {
    let req: Request<_> = TorrentStatisticsRequest {
        server_url: base(),
        request: StatisticsRequest {
            info_hash: INFO_HASH.to_owned(),
            file_idx: None,
        },
    }
    .into();
    assert_eq!(
        req.uri().to_string(),
        format!("https://host/stremio/{INFO_HASH}/stats.json")
    );
}
