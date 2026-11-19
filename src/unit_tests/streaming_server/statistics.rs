use futures::future;

use crate::{
    models::{common::Loadable, ctx::Ctx, streaming_server::StreamingServer},
    runtime::{
        msg::{Action, ActionStreamingServer, Internal, Msg},
        EnvFutureExt, UpdateWithCtx,
    },
    types::streaming_server::{Statistics, StatisticsRequest, TorrentStatisticsRequest},
    unit_tests::{TestEnv, FETCH_HANDLER},
};

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

#[test]
fn changing_the_statistics_request_clears_the_previous_files_statistics() {
    let _guard = TestEnv::reset().unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new(|_| future::pending().boxed_env());
    let ctx = Ctx::default();
    let (mut server, _) = StreamingServer::new::<TestEnv>(&ctx.profile);
    let url = server.selected.transport_url.clone();
    let first = StatisticsRequest {
        info_hash: INFO_HASH.to_owned(),
        file_idx: Some(0),
    };
    let second = StatisticsRequest {
        file_idx: Some(1),
        ..first.clone()
    };
    let statistics = Statistics {
        stream_name: Some("episode-1.mp4".to_owned()),
        ..serde_json::from_str(STATS_WITHOUT_FILE_INDEX).unwrap()
    };
    let request = |request: &StatisticsRequest| {
        Msg::Action(Action::StreamingServer(
            ActionStreamingServer::GetStatistics(request.clone()),
        ))
    };
    let response = |request: &StatisticsRequest, value| {
        Msg::Internal(Internal::StreamingServerStatisticsResult(
            (url.clone(), request.clone()),
            Ok(Some(value)),
        ))
    };
    let update = |server: &mut StreamingServer, msg| {
        <StreamingServer as UpdateWithCtx<TestEnv>>::update(server, &msg, &ctx)
    };

    update(&mut server, request(&first));
    assert_eq!(server.statistics, Some(Loadable::Loading));
    update(&mut server, response(&first, statistics.clone()));
    update(&mut server, request(&first));
    assert_eq!(server.statistics, Some(Loadable::Ready(statistics.clone())));

    update(&mut server, request(&second));
    assert_eq!(server.statistics, Some(Loadable::Loading));
    update(&mut server, response(&first, statistics.clone()));
    assert_eq!(server.statistics, Some(Loadable::Loading));

    let second_statistics = Statistics {
        stream_name: Some("episode-2.mp4".to_owned()),
        ..statistics
    };
    update(&mut server, response(&second, second_statistics.clone()));
    assert_eq!(server.statistics, Some(Loadable::Ready(second_statistics)));
}
