use std::any::Any;

use chrono::{TimeZone, Utc};
use futures::future;
use stremio_derive::Model;
use url::Url;

use crate::{
    constants::{CALENDAR_IDS_EXTRA_PROP, CATALOG_RESOURCE_NAME},
    models::{calendar::Calendar, ctx::Ctx},
    runtime::{
        msg::{Action, ActionLoad},
        Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestCatalog, ManifestExtra, ResourceResponse},
        library::{LibraryBucket, LibraryItem},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, SeriesInfo, Video},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW, REQUESTS},
};

fn library_item(id: &str, r#type: &str) -> LibraryItem {
    LibraryItem {
        id: id.to_string(),
        r#type: r#type.to_string(),
        removed: false,
        temp: false,
        name: Default::default(),
        poster: Default::default(),
        poster_shape: Default::default(),
        ctime: Some(TestEnv::now()),
        mtime: TestEnv::now(),
        state: Default::default(),
        behavior_hints: Default::default(),
    }
}

fn meta_item(id: &str, r#type: &str) -> MetaItem {
    MetaItem {
        preview: MetaItemPreview {
            id: id.to_string(),
            r#type: r#type.to_string(),
            ..MetaItemPreview::default()
        },
        videos: vec![Video {
            id: format!("{id}:1:2").to_owned(),
            released: Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
            series_info: Some(SeriesInfo {
                season: 1,
                episode: 2,
            }),
            ..Video::default()
        }],
    }
}

#[test]
fn calendar() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        calendar: Calendar,
    }

    let addon = Descriptor {
        transport_url: Url::parse("https://addon/manifest.json").unwrap(),
        flags: Default::default(),
        manifest: Manifest {
            id: "addon".to_owned(),
            types: vec!["series".into()],
            resources: vec![CATALOG_RESOURCE_NAME.into()],
            catalogs: vec![ManifestCatalog {
                id: "calendarVideosIds".to_owned(),
                r#type: "series".to_owned(),
                name: Some("calendar-videos".to_string()),
                extra: ManifestExtra::Full {
                    props: vec![CALENDAR_IDS_EXTRA_PROP.to_owned()],
                },
            }],
            ..Default::default()
        },
    };

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, method, .. }
                if url
                    == "https://addon/catalog/series/calendarVideosIds/calendarVideosIds=tt1.json"
                    && method == "GET" =>
            {
                future::ok(Box::new(ResourceResponse::MetasDetailed {
                    metas_detailed: vec![meta_item("tt1", "series")],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![addon],
                    ..Default::default()
                },
                library: LibraryBucket::new(None, vec![library_item("tt1", "series")]),
                ..Default::default()
            },
            calendar: Default::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::Calendar(None)),
        });
    });

    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "should have sent a request"
    );

    assert_eq!(
        runtime.model().unwrap().calendar.items[0].items.len(),
        1,
        "should have a calendar item"
    );
}
