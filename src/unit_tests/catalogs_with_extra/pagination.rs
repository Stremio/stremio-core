use std::any::Any;

use futures::future;
use serde_json::{from_value, json};
use stremio_derive::Model;

use crate::{
    models::{
        catalogs_with_extra::{CatalogsWithExtra, Selected},
        common::Loadable,
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionCatalogsWithExtra, ActionLoad},
        EnvFutureExt, Runtime, RuntimeAction,
    },
    types::{
        addon::{Descriptor, ResourceResponse},
        profile::{BoardSpotlight, Profile, Settings},
        resource::MetaItemPreview,
    },
    unit_tests::{default_fetch_handler, TestEnv, FETCH_HANDLER, REQUESTS},
};

#[test]
fn paginates_the_catalog_from_the_matching_addon_at_a_shared_url() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        board: CatalogsWithExtra,
    }

    let _guard = TestEnv::reset().unwrap();
    let descriptor = |id, catalog| -> Descriptor {
        from_value(json!({
            "transportUrl": "https://addon.example/manifest.json",
            "manifest": {
                "id": id,
                "name": id,
                "version": "1.0.0",
                "types": ["movie"],
                "resources": ["catalog"],
                "catalogs": [catalog],
            },
        }))
        .unwrap()
    };
    let first = descriptor("first", json!({ "id": "first", "type": "movie" }));
    let second = descriptor(
        "second",
        json!({ "id": "second", "type": "movie", "extraSupported": ["skip"] }),
    );
    const FIRST_URL: &str = "https://addon.example/catalog/movie/first.json";
    const SECOND_URL: &str = "https://addon.example/catalog/movie/second.json";
    const NEXT_PAGE_URL: &str = "https://addon.example/catalog/movie/second/skip=1.json";
    *FETCH_HANDLER.write().unwrap() = Box::new(|request| {
        let id = match request.url.as_str() {
            FIRST_URL => "tt1",
            SECOND_URL => "tt2",
            NEXT_PAGE_URL => "tt3",
            _ => return default_fetch_handler(request),
        };
        future::ok(Box::new(ResourceResponse::Metas {
            metas: vec![MetaItemPreview {
                id: id.to_owned(),
                ..Default::default()
            }],
        }) as Box<dyn Any + Send>)
        .boxed_env()
    });
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![first, second],
                    settings: Settings {
                        board_spotlight: BoardSpotlight {
                            enabled: false,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            board: CatalogsWithExtra::default(),
        },
        vec![],
        1000,
    );
    let dispatch = |action| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                action,
                field: None,
            })
        });
    };

    dispatch(Action::Load(ActionLoad::CatalogsWithExtra(Selected {
        r#type: None,
        extra: vec![],
    })));
    dispatch(Action::CatalogsWithExtra(
        ActionCatalogsWithExtra::LoadRange(0..1),
    ));
    dispatch(Action::CatalogsWithExtra(
        ActionCatalogsWithExtra::LoadNextPage(0),
    ));
    dispatch(Action::CatalogsWithExtra(
        ActionCatalogsWithExtra::LoadNextPage(1),
    ));

    let model = runtime.model().unwrap();
    assert_eq!(model.board.catalogs[0].len(), 1);
    assert_eq!(model.board.catalogs[1].len(), 2);
    assert!(matches!(
        &model.board.catalogs[1][1].content,
        Some(Loadable::Ready(items)) if items[0].id == "tt3"
    ));
    let requests = REQUESTS.read().unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[2].url, NEXT_PAGE_URL);
}
