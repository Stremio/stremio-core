use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::api::{APIResult, SuccessResponse};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::{LibraryBucket, LibraryItem, LibraryItemState};
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, User};
use crate::types::resource::{MetaItemBehaviorHints, MetaItemPreview, PosterShape};
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::types::True;
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW, REQUESTS, STORAGE,
};
use chrono::{TimeZone, Utc};
use futures::future;
use std::any::Any;
use stremio_derive::Model;
use url::Url;

#[test]
fn actionctx_addtolibrary() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastorePut"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id\",\"name\":\"name\",\"type\":\"type\",\"poster\":null,\"posterShape\":\"poster\",\"removed\":false,\"temp\":false,\"_ctime\":\"2020-01-01T00:00:00Z\",\"_mtime\":\"2020-01-01T00:00:00Z\",\"state\":{\"lastWatched\":\"2020-01-01T00:00:00Z\",\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"noNotif\":false},\"behaviorHints\":{\"defaultVideoId\":null,\"featuredVideoId\":null,\"hasScheduledVideos\":false}}]}" =>
            {
                future::ok(Box::new(APIResult::Ok(SuccessResponse { success: True {} },
                )) as Box<dyn Any + Send>).boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let meta_preview = MetaItemPreview {
        id: "id".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: None,
        background: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        links: vec![],
        trailer_streams: vec![],
        behavior_hints: Default::default(),
    };
    let library_item = LibraryItem {
        id: "id".to_owned(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: LibraryItemState {
            last_watched: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
            ..Default::default()
        },
        name: "name".to_owned(),
        r#type: "type".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    auth: Some(Auth {
                        key: AuthKey("auth_key".to_owned()),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: TestEnv::now(),
                            date_registered: TestEnv::now(),
                            trakt: None,
                            premium_expire: None,
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                                from: Some("tests".to_owned()),
                            },
                        },
                    }),
                    ..Default::default()
                },
                LibraryBucket {
                    uid: Some("id".to_owned()),
                    ..Default::default()
                },
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
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
                serde_json::from_str::<LibraryBucket>(data).unwrap()
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
        REQUESTS.read().unwrap().first().unwrap().url.to_owned(),
        "https://api.strem.io/api/datastorePut".to_owned(),
        "datastorePut request has been sent"
    );
}

#[test]
fn actionctx_addtolibrary_already_added() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let meta_preview = MetaItemPreview {
        id: "id".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: Some(Url::parse("http://poster").unwrap()),
        background: None,
        poster_shape: PosterShape::Square,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        links: vec![],
        trailer_streams: vec![],
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("video_id2".to_owned()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let library_item = LibraryItem {
        id: "id".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: Some(Url::parse("http://poster").unwrap()),
        poster_shape: PosterShape::Square,
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 2, 0, 0, 0).unwrap(),
        state: LibraryItemState {
            video_id: Some("video_id".to_owned()),
            ..LibraryItemState::default()
        },
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("video_id2".to_owned()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2020, 1, 2, 0, 0, 0).unwrap();
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile::default(),
                LibraryBucket {
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
                            ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                            mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
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
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
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
                serde_json::from_str::<LibraryBucket>(data).unwrap()
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
