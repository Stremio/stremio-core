use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Effects, Env, EnvFuture, Runtime, RuntimeAction};
use crate::types::api::{APIResult, SuccessResponse};
use crate::types::library::{
    LibraryBucket, LibraryItem, LibraryItemBehaviorHints, LibraryItemState,
};
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, User};
use crate::types::resource::{MetaItemBehaviorHints, MetaItemPreview, PosterShape};
use crate::types::True;
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW, REQUESTS, STORAGE,
};
use chrono::prelude::TimeZone;
use chrono::Utc;
use futures::{future, FutureExt};
use std::any::Any;
use stremio_derive::Model;

#[test]
fn actionctx_addtolibrary() {
    #[derive(Model, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastorePut"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id\",\"name\":\"name\",\"type\":\"type\",\"poster\":null,\"posterShape\":\"poster\",\"removed\":false,\"temp\":false,\"_ctime\":\"2020-01-01T00:00:00Z\",\"_mtime\":\"2020-01-01T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"lastVidReleased\":null,\"noNotif\":false},\"behaviorHints\":{\"defaultVideoId\":null}}]}" =>
            {
                future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>).boxed_local()
            }
            _ => default_fetch_handler(request),
        }
    }
    let meta_preview = MetaItemPreview {
        id: "id".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        trailer_streams: vec![],
        behavior_hints: Default::default(),
    };
    let library_item = LibraryItem {
        id: "id".to_owned(),
        removed: false,
        temp: false,
        ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
        state: Default::default(),
        name: "name".to_owned(),
        r#type: "type".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    TestEnv::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    auth: Some(Auth {
                        key: AuthKey("auth_key".to_owned()),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: TestEnv::now(),
                            date_registered: TestEnv::now(),
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                            },
                        },
                    }),
                    ..Default::default()
                },
                library: LibraryBucket {
                    uid: Some("id".to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        Effects::none().unchanged(),
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::AddToLibrary(meta_preview.to_owned())),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.library.items.len(),
        1,
        "There is one library item in memory"
    );
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .library
            .items
            .get(&meta_preview.id),
        Some(&library_item),
        "Library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibraryBucket>(&data).unwrap()
                    == LibraryBucket::new(Some("id".to_owned()), vec![library_item])
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
    #[derive(Model, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let meta_preview = MetaItemPreview {
        id: "id".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: Some("poster".to_owned()),
        poster_shape: PosterShape::Square,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        trailer_streams: vec![],
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("video_id2".to_owned()),
            featured_video_id: None,
            has_scheduled_videos: false,
        },
    };
    let library_item = LibraryItem {
        id: "id".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: Some("poster".to_owned()),
        poster_shape: PosterShape::Square,
        removed: false,
        temp: false,
        ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0),
        state: LibraryItemState {
            video_id: Some("video_id".to_owned()),
            ..LibraryItemState::default()
        },
        behavior_hints: LibraryItemBehaviorHints {
            default_video_id: Some("video_id2".to_owned()),
        },
    };
    TestEnv::reset();
    *NOW.write().unwrap() = Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![(
                        "id".to_owned(),
                        LibraryItem {
                            id: "id".to_owned(),
                            r#type: "typename_".to_owned(),
                            name: "name_".to_owned(),
                            poster: None,
                            poster_shape: PosterShape::Poster,
                            removed: true,
                            temp: true,
                            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
                            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                            state: LibraryItemState {
                                video_id: Some("video_id".to_owned()),
                                ..LibraryItemState::default()
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
        Effects::none().unchanged(),
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::AddToLibrary(meta_preview.to_owned())),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.library.items.len(),
        1,
        "There is one library item in memory"
    );
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .library
            .items
            .get(&library_item.id),
        Some(&library_item),
        "Library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibraryBucket>(&data).unwrap()
                    == LibraryBucket::new(None, vec![library_item])
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
