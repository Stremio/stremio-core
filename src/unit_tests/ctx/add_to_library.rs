use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{APIResult, SuccessResponse, True};
use crate::types::library::{LibBucket, LibItem, LibItemBehaviorHints, LibItemState};
use crate::types::profile::{Auth, GDPRConsent, Profile, User};
use crate::types::resource::{MetaItemBehaviorHints, MetaItemPreview, PosterShape};
use crate::unit_tests::{
    default_fetch_handler, Env, Request, FETCH_HANDLER, NOW, REQUESTS, STORAGE,
};
use chrono::prelude::TimeZone;
use chrono::Utc;
use futures::future;
use std::any::Any;
use std::fmt::Debug;
use stremio_derive::Model;
use tokio::runtime::current_thread::run;

#[test]
fn actionctx_addtolibrary() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastorePut"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id\",\"name\":\"name\",\"type\":\"type_name\",\"poster\":null,\"posterShape\":\"poster\",\"removed\":false,\"temp\":false,\"_ctime\":\"2020-01-01T00:00:00Z\",\"_mtime\":\"2020-01-01T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"lastVidReleased\":null,\"noNotif\":false},\"behaviorHints\":{\"defaultVideoId\":null}}]}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    let meta_preview = MetaItemPreview {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        trailers: vec![],
        behavior_hints: Default::default(),
    };
    let lib_item = LibItem {
        id: "id".to_owned(),
        removed: false,
        temp: false,
        ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
        state: Default::default(),
        name: "name".to_owned(),
        type_name: "type_name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    auth: Some(Auth {
                        key: "auth_key".to_owned(),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: Env::now(),
                            date_registered: Env::now(),
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                                from: "tests".to_owned(),
                            },
                        },
                    }),
                    ..Default::default()
                },
                library: LibBucket {
                    uid: Some("id".to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
            meta_preview.to_owned(),
        )))),
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.library.items.len(),
        1,
        "There is one library item in memory"
    );
    assert_eq!(
        runtime
            .app
            .read()
            .unwrap()
            .ctx
            .library
            .items
            .get(&meta_preview.id),
        Some(&lib_item),
        "Library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibBucket>(&data).unwrap()
                    == LibBucket::new(Some("id".to_owned()), vec![lib_item])
            }),
        "Library recent slot updated successfully in storage"
    );
    assert!(
        STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).is_none(),
        "Library slot updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().url.to_owned(),
        "https://api.strem.io/api/datastorePut".to_owned(),
        "datastorePut request has been sent"
    );
}

#[test]
fn actionctx_addtolibrary_already_added() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let meta_preview = MetaItemPreview {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: Some("poster".to_owned()),
        poster_shape: PosterShape::Square,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        trailers: vec![],
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("video_id2".to_owned()),
            featured_video_id: None,
        },
    };
    let lib_item = LibItem {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: Some("poster".to_owned()),
        poster_shape: PosterShape::Square,
        removed: false,
        temp: false,
        ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0),
        state: LibItemState {
            video_id: Some("video_id".to_owned()),
            ..LibItemState::default()
        },
        behavior_hints: LibItemBehaviorHints {
            default_video_id: Some("video_id2".to_owned()),
        },
    };
    Env::reset();
    *NOW.write().unwrap() = Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0);
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                library: LibBucket {
                    uid: None,
                    items: vec![(
                        "id".to_owned(),
                        LibItem {
                            id: "id".to_owned(),
                            type_name: "type_name_".to_owned(),
                            name: "name_".to_owned(),
                            poster: None,
                            poster_shape: PosterShape::Poster,
                            removed: true,
                            temp: true,
                            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
                            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                            state: LibItemState {
                                video_id: Some("video_id".to_owned()),
                                ..LibItemState::default()
                            },
                            behavior_hints: Default::default(),
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
            meta_preview.to_owned(),
        )))),
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.library.items.len(),
        1,
        "There is one library item in memory"
    );
    assert_eq!(
        runtime
            .app
            .read()
            .unwrap()
            .ctx
            .library
            .items
            .get(&lib_item.id),
        Some(&lib_item),
        "Library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibBucket>(&data).unwrap()
                    == LibBucket::new(None, vec![lib_item])
            }),
        "Library recent slot updated successfully in storage"
    );
    assert!(
        STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).is_none(),
        "Library slot updated successfully in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
