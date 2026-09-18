use std::any::Any;

use chrono::{Duration, TimeZone, Utc};
use futures::future;
use stremio_derive::Model;
use url::Url;

use crate::{
    constants::CATALOG_RESOURCE_NAME,
    models::{ctx::Ctx, live_tv_guide::LiveTvGuide},
    runtime::{
        msg::{Action, ActionLiveTvGuide, ActionLoad},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestBehaviorHints, ResourceResponse},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, Video, VideoEpgInfo},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW, REQUESTS},
};

#[test]
fn live_tv_guide_refresh_live() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        live_tv_guide: LiveTvGuide,
    }

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, method, .. }
                if url == "https://addon/catalog/tv/guide/date=2026-07-02.json"
                    && method == "GET" =>
            {
                future::ok(Box::new(ResourceResponse::MetasDetailed {
                    metas_detailed: vec![MetaItem {
                        preview: MetaItemPreview {
                            id: "pure:axn".to_owned(),
                            r#type: "tv".to_owned(),
                            name: "AXN".to_owned(),
                            ..MetaItemPreview::default()
                        },
                        videos: vec![Video {
                            id: "pure:axn:1".to_owned(),
                            epg_info: Some(VideoEpgInfo {
                                start_time: Utc.with_ymd_and_hms(2026, 7, 2, 11, 0, 0).unwrap(),
                                end_time: Utc.with_ymd_and_hms(2026, 7, 2, 12, 0, 0).unwrap(),
                                runtime: None,
                                release_info: None,
                                genres: vec![],
                                cast: vec![],
                                directors: vec![],
                                links: vec![],
                                ratings: vec![],
                            }),
                            ..Video::default()
                        }],
                    }],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 7, 2, 11, 30, 0).unwrap();

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![Descriptor {
                        transport_url: Url::parse("https://addon/manifest.json").unwrap(),
                        flags: Default::default(),
                        manifest: Manifest {
                            id: "addon".to_owned(),
                            types: vec!["tv".into()],
                            resources: vec![CATALOG_RESOURCE_NAME.into()],
                            catalogs: vec![serde_json::from_value(serde_json::json!({
                                "id": "guide", "type": "tv", "name": "PureTV",
                                "extra": [{ "name": "date" }],
                            }))
                            .unwrap()],
                            behavior_hints: ManifestBehaviorHints {
                                epg_provider: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            live_tv_guide: Default::default(),
        },
        vec![],
        1000,
    );
    let dispatch = |action: Action| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action,
            });
        });
    };

    dispatch(Action::Load(ActionLoad::LiveTvGuide(None)));
    assert_eq!(REQUESTS.read().unwrap().len(), 1);
    assert_eq!(runtime.model().unwrap().live_tv_guide.channels.len(), 1);

    dispatch(Action::LiveTvGuide(ActionLiveTvGuide::RefreshLive));
    dispatch(Action::Load(ActionLoad::LiveTvGuide(None)));
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "a fresh guide is neither refreshed nor reloaded for the same selection"
    );

    *NOW.write().unwrap() += Duration::minutes(15);
    dispatch(Action::Load(ActionLoad::LiveTvGuide(None)));
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "loading the same selection does not refresh a stale guide"
    );

    dispatch(Action::LiveTvGuide(ActionLiveTvGuide::RefreshLive));
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        2,
        "a refresh reloads the stale guide pages"
    );
    assert_eq!(runtime.model().unwrap().live_tv_guide.channels.len(), 1);

    dispatch(Action::LiveTvGuide(ActionLiveTvGuide::RefreshLive));
    assert_eq!(REQUESTS.read().unwrap().len(), 2);
}
