use std::any::Any;

use chrono::{TimeZone, Utc};
use futures::future;
use stremio_derive::Model;
use url::Url;

use crate::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        ctx::Ctx,
        live_tv_continue_watching::LiveTvContinueWatching,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionCtx, ActionLoad, ActionPlayer},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{
            Descriptor, Manifest, ManifestBehaviorHints, ResourcePath, ResourceRequest,
            ResourceResponse,
        },
        library::LibraryBucket,
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, Stream, StreamSource, Video, VideoEpgInfo},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW},
};

const CHANNEL_ID: &str = "pure:axn";
const ADDON_URL: &str = "https://addon/manifest.json";

fn epg_addon() -> Descriptor {
    Descriptor {
        transport_url: Url::parse(ADDON_URL).unwrap(),
        flags: Default::default(),
        manifest: Manifest {
            id: "addon".to_owned(),
            types: vec!["tv".into()],
            resources: vec![META_RESOURCE_NAME.into(), STREAM_RESOURCE_NAME.into()],
            id_prefixes: Some(vec!["pure:".to_owned()]),
            behavior_hints: ManifestBehaviorHints {
                epg_provider: true,
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

fn channel_meta() -> MetaItem {
    MetaItem {
        preview: MetaItemPreview {
            id: CHANNEL_ID.to_owned(),
            r#type: "tv".to_owned(),
            name: "AXN".to_owned(),
            ..MetaItemPreview::default()
        },
        videos: vec![Video {
            id: format!("{CHANNEL_ID}:1"),
            epg_info: Some(VideoEpgInfo {
                start_time: Utc.with_ymd_and_hms(2026, 7, 2, 12, 0, 0).unwrap(),
                end_time: Utc.with_ymd_and_hms(2026, 7, 2, 13, 0, 0).unwrap(),
                runtime: None,
                release_info: None,
                genres: vec![],
                cast: vec![],
                directors: vec![],
                links: vec![],
            }),
            ..Video::default()
        }],
    }
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match &request {
        // meta for the channel - used both by the player load and by the
        // Live TV row load
        Request { url, .. } if url == "https://addon/meta/tv/pure%3Aaxn.json" => {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: channel_meta(),
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        // stream for the live channel
        Request { url, .. } if url == "https://addon/stream/tv/pure%3Aaxn.json" => future::ok(
            Box::new(ResourceResponse::Streams { streams: vec![] }) as Box<dyn Any + Send>,
        )
        .boxed_env(),
        _ => default_fetch_handler(request),
    }
}

fn meta_request() -> ResourceRequest {
    ResourceRequest {
        base: Url::parse(ADDON_URL).unwrap(),
        path: ResourcePath::without_extra(META_RESOURCE_NAME, "tv", CHANNEL_ID),
    }
}

fn stream_request() -> ResourceRequest {
    ResourceRequest {
        base: Url::parse(ADDON_URL).unwrap(),
        path: ResourcePath::without_extra(STREAM_RESOURCE_NAME, "tv", CHANNEL_ID),
    }
}

fn live_stream() -> Stream {
    Stream {
        source: StreamSource::Url {
            url: "https://source_url".parse().unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    }
}

#[test]
fn play_live_channel_then_board_shows_it() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 7, 2, 12, 30, 0).unwrap();

    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
        live_tv_continue_watching: LiveTvContinueWatching,
    }

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![epg_addon()],
                    ..Default::default()
                },
                // start empty: the player creates the library item on play
                library: LibraryBucket::default(),
                ..Default::default()
            },
            player: Player::default(),
            live_tv_continue_watching: Default::default(),
        },
        vec![],
        1000,
    );

    // 1. play the live channel
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::Player(Box::new(Selected {
                stream: live_stream(),
                stream_request: Some(stream_request()),
                meta_request: Some(meta_request()),
                subtitles_path: None,
            }))),
        });
    });

    // 2. it plays for some time - a live stream reports no duration
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::TimeChanged {
                time: 600_000,
                duration: 0,
                device: "test_device".to_owned(),
            }),
        });
    });

    // 3. back to the board - the player unloads
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Unload,
        });
    });

    // the channel is now in the library, watched, but with time_offset zeroed
    {
        let model = runtime.model().unwrap();
        let library_item = model
            .ctx
            .library
            .items
            .get(CHANNEL_ID)
            .expect("the played channel should be in the library");
        assert_eq!(
            library_item.state.time_offset, 0,
            "the player zeroes a live channel's time_offset on unload (duration == 0)",
        );
        assert!(
            library_item.state.last_watched.is_some(),
            "the channel must carry a last_watched timestamp",
        );
    }

    // 4. the board mounts the Live TV row
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::LiveTvContinueWatching),
        });
    });

    let model = runtime.model().unwrap();
    let live_tv = &model.live_tv_continue_watching;
    assert_eq!(
        live_tv
            .items
            .iter()
            .map(|item| item.channel.id.as_str())
            .collect::<Vec<_>>(),
        vec![CHANNEL_ID],
        "the just-watched live channel must appear in the Live TV row despite time_offset == 0",
    );
    assert_eq!(
        live_tv.items[0].channel.name, "AXN",
        "the channel preview is filled from the fetched meta",
    );
    assert_eq!(
        live_tv.items[0]
            .shows
            .iter()
            .map(|show| show.id.as_str())
            .collect::<Vec<_>>(),
        vec!["pure:axn:1"],
        "the channel's program shows are fetched from the epgProvider addon",
    );
    drop(model);

    // 5. dismiss the channel - RemoveFromLibrary clears `temp`, which is what
    // drops it from the row (a rewind would only zero the already-zero
    // time_offset the row ignores)
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::RemoveFromLibrary(CHANNEL_ID.to_owned())),
        });
    });

    let model = runtime.model().unwrap();
    assert!(
        model.live_tv_continue_watching.items.is_empty(),
        "dismissing the channel (RemoveFromLibrary) removes it from the Live TV row",
    );
}
