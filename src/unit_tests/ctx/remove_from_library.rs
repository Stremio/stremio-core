use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{APIResult, SuccessResponse, True};
use crate::types::library::{LibBucket, LibItem};
use crate::types::profile::{Auth, GDPRConsent, Profile, User, UID};
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
fn actionctx_removefromlibrary() {
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
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id\",\"name\":\"name\",\"type\":\"type_name\",\"poster\":null,\"posterShape\":\"poster\",\"removed\":true,\"temp\":false,\"_ctime\":\"2020-01-01T00:00:00Z\",\"_mtime\":\"2020-01-02T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"lastVidReleased\":null,\"noNotif\":false},\"behaviorHints\":{\"defaultVideoId\":null}}]}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
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
    let lib_item_removed = LibItem {
        removed: true,
        mtime: Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0),
        ..lib_item.to_owned()
    };
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0);
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string::<(UID, Vec<LibItem>)>(&(
            Some("id".to_owned()),
            vec![lib_item.to_owned()],
        ))
        .unwrap(),
    );
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
                                time: Env::now(),
                                from: "tests".to_owned(),
                            },
                        },
                    }),
                    ..Default::default()
                },
                library: LibBucket {
                    uid: Some("id".to_owned()),
                    items: vec![("id".to_owned(), lib_item.to_owned())]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::RemoveFromLibrary(
            lib_item.id.to_owned(),
        )))),
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
        Some(&lib_item_removed),
        "Library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<(UID, Vec<LibItem>)>(&data).unwrap()
                    == (Some("id".to_owned()), vec![lib_item_removed])
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
fn actionctx_removefromlibrary_not_added() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
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
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string::<(UID, Vec<LibItem>)>(&(None, vec![lib_item.to_owned()])).unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                library: LibBucket {
                    uid: None,
                    items: vec![("id".to_owned(), lib_item.to_owned())]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::RemoveFromLibrary(
            "id2".to_owned(),
        )))),
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
        "Library not updated in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<(UID, Vec<LibItem>)>(&data).unwrap()
                    == (None, vec![lib_item])
            }),
        "Library recent slot not updated in storage"
    );
    assert!(
        STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).is_none(),
        "Library slot not updated in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
