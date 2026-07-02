use std::any::Any;

use chrono::{TimeZone, Utc};
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
        addon::{Descriptor, ExtraValue, Manifest, ManifestBehaviorHints, ResourceResponse},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, Video, VideoEpgInfo},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW, REQUESTS},
};

fn epg_info(start: (u32, u32), end: (u32, u32)) -> VideoEpgInfo {
    VideoEpgInfo {
        start_time: Utc
            .with_ymd_and_hms(2026, 7, 2, start.0, start.1, 0)
            .unwrap(),
        end_time: Utc.with_ymd_and_hms(2026, 7, 2, end.0, end.1, 0).unwrap(),
        runtime: None,
        release_info: None,
        genres: vec![],
        cast: vec![],
        directors: vec![],
        links: vec![],
    }
}

fn channel_meta_item() -> MetaItem {
    MetaItem {
        preview: MetaItemPreview {
            id: "pure:axn".to_owned(),
            r#type: "tv".to_owned(),
            name: "AXN".to_owned(),
            ..MetaItemPreview::default()
        },
        videos: vec![
            // out of order on purpose - shows must be sorted by start time
            Video {
                id: "pure:axn:2".to_owned(),
                epg_info: Some(epg_info((12, 0), (13, 0))),
                ..Video::default()
            },
            Video {
                id: "pure:axn:1".to_owned(),
                epg_info: Some(epg_info((11, 0), (12, 0))),
                ..Video::default()
            },
            // no epg info - not a program show, must be filtered out
            Video {
                id: "pure:axn:trailer".to_owned(),
                ..Video::default()
            },
            // previous day - must be filtered out
            Video {
                id: "pure:axn:0".to_owned(),
                epg_info: Some(VideoEpgInfo {
                    start_time: Utc.with_ymd_and_hms(2026, 7, 1, 11, 0, 0).unwrap(),
                    end_time: Utc.with_ymd_and_hms(2026, 7, 1, 12, 0, 0).unwrap(),
                    ..epg_info((11, 0), (12, 0))
                }),
                ..Video::default()
            },
        ],
    }
}

#[test]
fn live_tv_guide() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        live_tv_guide: LiveTvGuide,
    }

    let addon = Descriptor {
        transport_url: Url::parse("https://addon/manifest.json").unwrap(),
        flags: Default::default(),
        manifest: Manifest {
            id: "addon".to_owned(),
            types: vec!["tv".into()],
            resources: vec![CATALOG_RESOURCE_NAME.into()],
            catalogs: vec![serde_json::from_value(
                // the catalog does NOT declare the `date` extra -
                // epgProvider implies support for it;
                // pagination is opt-in via the `skip` extra
                serde_json::json!({
                    "id": "guide", "type": "tv", "name": "PureTV",
                    "extra": [{ "name": "skip" }],
                }),
            )
            .unwrap()],
            behavior_hints: ManifestBehaviorHints {
                epg_provider: true,
                ..Default::default()
            },
            ..Default::default()
        },
    };

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, method, .. }
                if url == "https://addon/catalog/tv/guide/date=2026-07-02.json"
                    && method == "GET" =>
            {
                future::ok(Box::new(ResourceResponse::MetasDetailed {
                    metas_detailed: vec![channel_meta_item()],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, method, .. }
                if url == "https://addon/catalog/tv/guide/skip=1&date=2026-07-02.json"
                    && method == "GET" =>
            {
                future::ok(Box::new(ResourceResponse::MetasDetailed {
                    metas_detailed: vec![MetaItem {
                        preview: MetaItemPreview {
                            id: "pure:amc".to_owned(),
                            r#type: "tv".to_owned(),
                            name: "AMC".to_owned(),
                            ..MetaItemPreview::default()
                        },
                        videos: vec![Video {
                            id: "pure:amc:1".to_owned(),
                            epg_info: Some(epg_info((11, 25), (13, 35))),
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
                    addons: vec![addon],
                    ..Default::default()
                },
                ..Default::default()
            },
            live_tv_guide: Default::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::LiveTvGuide(None)),
        });
    });

    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "should have sent the guide request with the date extra appended"
    );

    // drop the model lock guard before the next dispatch -
    // holding it would deadlock the runtime
    {
        let model = runtime.model().unwrap();
        let live_tv_guide = &model.live_tv_guide;
        let selected = live_tv_guide.selected.as_ref().expect("should be selected");
        assert_eq!(
            selected.date,
            Some(chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap()),
            "date should default to today"
        );
        assert_eq!(
            live_tv_guide.selectable.catalogs.len(),
            1,
            "should have a selectable catalog"
        );
        assert!(
            live_tv_guide.selectable.catalogs[0].selected,
            "the guide catalog should be selected"
        );
        assert_eq!(
            live_tv_guide.channels.len(),
            1,
            "should have a channel guide"
        );
        let channel_guide = &live_tv_guide.channels[0];
        assert_eq!(channel_guide.channel.id, "pure:axn");
        assert_eq!(
            channel_guide
                .shows
                .iter()
                .map(|show| show.id.as_str())
                .collect::<Vec<_>>(),
            vec!["pure:axn:1", "pure:axn:2"],
            "should include only the selected date's shows, ordered by start time"
        );
        let next_page = live_tv_guide
            .selectable
            .next_page
            .as_ref()
            .expect("should have a next page - the catalog declares the skip extra");
        assert_eq!(
            next_page.request.path.extra,
            vec![
                ExtraValue {
                    name: "skip".to_owned(),
                    value: "1".to_owned(),
                },
                ExtraValue {
                    name: "date".to_owned(),
                    value: "2026-07-02".to_owned(),
                },
            ],
            "the next page request should carry the date and skip extras"
        );
    }

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::LiveTvGuide(ActionLiveTvGuide::LoadNextPage),
        });
    });

    assert_eq!(
        REQUESTS.read().unwrap().len(),
        2,
        "should have sent the next page request"
    );

    let live_tv_guide = &runtime.model().unwrap().live_tv_guide;
    assert_eq!(
        live_tv_guide
            .channels
            .iter()
            .map(|channel_guide| channel_guide.channel.id.as_str())
            .collect::<Vec<_>>(),
        vec!["pure:axn", "pure:amc"],
        "should append the next page's channels"
    );
    assert_eq!(
        live_tv_guide
            .selectable
            .next_page
            .as_ref()
            .expect("should still have a next page")
            .request
            .path
            .get_extra_first_value("skip"),
        Some(&"2".to_owned()),
        "the skip extra should count all loaded channels"
    );
}
