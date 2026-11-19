use chrono::Duration;
use futures::future;
use serde_json::json;
use stremio_derive::Model;

use crate::{
    models::{
        ctx::Ctx,
        library_with_filters::{
            ContinueWatchingFilter, LibraryRequest, LibraryWithFilters, Selected,
        },
    },
    runtime::{
        msg::{Action, ActionCtx, ActionLoad},
        Env, EnvFutureExt, Runtime, RuntimeAction,
    },
    types::{
        addon::{Descriptor, ResourceResponse},
        library::{LibraryBucket, LibraryItem, LibraryItemState},
        profile::Profile,
    },
    unit_tests::{TestEnv, FETCH_HANDLER},
};

#[test]
fn continue_watching_refreshes_after_pulling_episode_notifications() {
    #[derive(Model, Clone)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        continue_watching: LibraryWithFilters<ContinueWatchingFilter>,
    }

    let _guard = TestEnv::reset().unwrap();
    let now = TestEnv::now();
    let addon: Descriptor = serde_json::from_value(json!({
        "transportUrl": "https://addon.example/manifest.json",
        "manifest": {
            "id": "episodes", "name": "Episodes", "version": "1.0.0",
            "types": ["series"], "resources": ["catalog"],
            "catalogs": [{
                "id": "recent", "type": "series",
                "extra": [{"name": "lastVideosIds", "isRequired": true}]
            }]
        }
    }))
    .unwrap();
    let item = LibraryItem {
        id: "tt1".to_owned(),
        name: "Show".to_owned(),
        r#type: "series".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: now - Duration::days(2),
        state: LibraryItemState {
            last_watched: Some(now - Duration::days(2)),
            ..Default::default()
        },
        behavior_hints: Default::default(),
    };
    let response: ResourceResponse = serde_json::from_value(json!({
        "metasDetailed": [{
            "id": "tt1", "type": "series", "name": "Show",
            "videos": [{
                "id": "tt1:1:2", "title": "Episode 2", "season": 1, "episode": 2,
                "released": now - Duration::days(1)
            }]
        }]
    }))
    .unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new(move |request| {
        assert_eq!(
            request.url,
            "https://addon.example/catalog/series/recent/lastVideosIds=tt1.json"
        );
        future::ok(Box::new(response.clone()) as Box<dyn std::any::Any + Send>).boxed_env()
    });
    let library = LibraryBucket::new(None, vec![item.clone()]);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![addon],
                    ..Default::default()
                },
                library: library.clone(),
                ..Default::default()
            },
            continue_watching: LibraryWithFilters::default(),
        },
        vec![],
        1000,
    );
    let dispatch = |action| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action,
            })
        })
    };
    dispatch(Action::Load(ActionLoad::LibraryWithFilters(Selected {
        request: LibraryRequest {
            r#type: None,
            sort: Default::default(),
            page: Default::default(),
        },
    })));
    assert!(runtime
        .model()
        .unwrap()
        .continue_watching
        .catalog
        .is_empty());

    dispatch(Action::Ctx(ActionCtx::PullNotifications));

    let model = runtime.model().unwrap();
    assert!(model.ctx.notifications.items["tt1"].contains_key("tt1:1:2"));
    assert_eq!(model.ctx.library, library);
    assert_eq!(model.continue_watching.catalog, vec![item]);
    assert!(model
        .continue_watching
        .selectable
        .types
        .iter()
        .any(|entry| entry.r#type.as_deref() == Some("series")));
}
