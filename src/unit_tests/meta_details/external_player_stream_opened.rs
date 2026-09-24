use crate::{
    constants::{CINEMETA_URL, META_RESOURCE_NAME, OFFICIAL_ADDONS, STREAM_RESOURCE_NAME},
    models::{
        ctx::Ctx,
        meta_details::{MetaDetails, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionMetaDetails},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{ResourcePath, ResourceResponse},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, Stream, StreamSource, Video},
        streams::{StreamsBucket, StreamsItem, StreamsItemKey},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER},
};
use chrono::{DateTime, Utc};
use futures::future;
use std::any::Any;
use stremio_derive::Model;
use url::Url;

#[derive(Model, Default, Clone, Debug)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    meta_details: MetaDetails,
}

fn create_stream(url: &str) -> Stream {
    Stream {
        source: StreamSource::Url {
            url: Url::parse(url).unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    }
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url == "https://v3-cinemeta.strem.io/meta/series/tt123456.json" => {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: MetaItem {
                    preview: MetaItemPreview {
                        id: "tt123456".to_owned(),
                        r#type: "series".to_owned(),
                        ..Default::default()
                    },
                    videos: vec![Video {
                        id: "tt123456:1:1".to_owned(),
                        streams: vec![
                            create_stream("https://example.com/old.mkv"),
                            create_stream("https://example.com/new.mkv"),
                        ],
                        ..Default::default()
                    }],
                },
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

#[test]
fn external_player_stream_opened_replaces_last_used_stream() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let key = StreamsItemKey {
        meta_id: "tt123456".to_owned(),
        video_id: "tt123456:1:1".to_owned(),
    };
    let old_streams_item = StreamsItem {
        stream: create_stream("https://example.com/old.mkv"),
        r#type: "series".to_owned(),
        meta_id: key.meta_id.to_owned(),
        video_id: key.video_id.to_owned(),
        meta_transport_url: CINEMETA_URL.to_owned(),
        stream_transport_url: CINEMETA_URL.to_owned(),
        state: None,
        mtime: DateTime::<Utc>::default(),
    };
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: OFFICIAL_ADDONS
                        .iter()
                        .filter(|addon| addon.transport_url == *CINEMETA_URL)
                        .cloned()
                        .collect(),
                    ..Default::default()
                },
                streams: StreamsBucket {
                    uid: None,
                    items: vec![(key.to_owned(), old_streams_item)]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
            meta_details: Default::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::MetaDetails(Selected {
                meta_path: ResourcePath {
                    resource: META_RESOURCE_NAME.to_owned(),
                    r#type: "series".to_owned(),
                    id: "tt123456".to_owned(),
                    extra: vec![],
                },
                stream_path: Some(ResourcePath {
                    resource: STREAM_RESOURCE_NAME.to_owned(),
                    r#type: "series".to_owned(),
                    id: "tt123456:1:1".to_owned(),
                    extra: vec![],
                }),
                guess_stream: false,
            })),
        });
    });
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::MetaDetails(ActionMetaDetails::ExternalPlayerStreamOpened(
                create_stream("https://example.com/new.mkv"),
            )),
        });
    });

    let model = runtime.model().unwrap();
    assert_eq!(
        model.ctx.streams.items.get(&key).unwrap().stream,
        create_stream("https://example.com/new.mkv"),
        "the externally opened stream should replace the last used stream",
    );
}
