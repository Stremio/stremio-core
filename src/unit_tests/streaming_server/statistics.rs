use crate::types::streaming_server::{Statistics, StatisticsRequest, TorrentStatisticsRequest};

const INFO_HASH: &str = "120c34efc4b29192d7c504344a39a7b0c2c6da21";
// Recorded from server.js v4.21.1 for a single-file torrent, trimmed to the fields Core reads.
const STATS_WITHOUT_FILE_INDEX: &str = r#"{"name":"video.mp4","infoHash":"120c34efc4b29192d7c504344a39a7b0c2c6da21","files":[{"path":"video.mp4","name":"video.mp4","length":1985917,"offset":0}],"sources":[{"numFound":0,"numFoundUniq":0,"numRequests":1,"url":"tracker:http://127.0.0.1:1/announce","lastStarted":"2026-09-27T18:37:32.826Z"},{"numFound":0,"numFoundUniq":0,"numRequests":1,"url":"dht:120c34efc4b29192d7c504344a39a7b0c2c6da21","lastStarted":"2026-09-27T18:37:32.826Z"}],"opts":{"connections":55,"dht":false,"growler":{"flood":0,"pulse":3670016},"handshakeTimeout":20000,"path":"/stremio-cache/120c34efc4b29192d7c504344a39a7b0c2c6da21","peerSearch":{"min":40,"max":150,"sources":["tracker:udp://zer0day.ch:1337/announce","tracker:udp://tracker.publictracker.xyz:6969/announce"]},"swarmCap":{"minPeers":5,"maxSpeed":2621440},"timeout":4000,"tracker":false,"virtual":true},"downloadSpeed":0,"uploadSpeed":0,"downloaded":0,"uploaded":0,"unchoked":0,"peers":0,"queued":0,"unique":0,"connectionTries":0,"peerSearchRunning":true,"swarmConnections":0,"swarmPaused":false,"swarmSize":55}"#;

fn statistics_url(file_idx: Option<u16>) -> String {
    let request: http::Request<()> = TorrentStatisticsRequest {
        server_url: "http://127.0.0.1:11470/".parse().unwrap(),
        request: StatisticsRequest {
            info_hash: INFO_HASH.to_owned(),
            file_idx,
        },
    }
    .into();
    request.uri().to_string()
}

#[test]
fn statistics_url_with_file_index() {
    assert_eq!(
        statistics_url(Some(7)),
        format!("http://127.0.0.1:11470/{INFO_HASH}/7/stats.json")
    );
}

#[test]
fn statistics_url_without_file_index() {
    assert_eq!(
        statistics_url(None),
        format!("http://127.0.0.1:11470/{INFO_HASH}/stats.json")
    );
}

#[test]
fn statistics_request_file_index_is_optional() {
    for (json, file_idx) in [
        (
            format!(r#"{{"infoHash":"{INFO_HASH}","fileIdx":3}}"#),
            Some(3),
        ),
        (
            format!(r#"{{"infoHash":"{INFO_HASH}","fileIdx":null}}"#),
            None,
        ),
        (format!(r#"{{"infoHash":"{INFO_HASH}"}}"#), None),
    ] {
        let request = serde_json::from_str::<StatisticsRequest>(&json).unwrap();
        assert_eq!(request.file_idx, file_idx, "{json}");
    }
}

#[test]
fn statistics_without_file_index() {
    let statistics = serde_json::from_str::<Statistics>(STATS_WITHOUT_FILE_INDEX).unwrap();
    assert_eq!(statistics.info_hash, INFO_HASH);
    assert_eq!(statistics.files.len(), 1);
    assert_eq!(statistics.stream_len, None);
    assert_eq!(statistics.stream_name, None);
    assert_eq!(statistics.stream_progress, None);
}
