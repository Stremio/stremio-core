use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{APIResult, Auth, SuccessResponse, True, User};
use crate::types::profile::{Profile, UID};
use crate::types::{LibBucket, LibItem};
use crate::unit_tests::{default_fetch_handler, Env, Request, FETCH_HANDLER, REQUESTS, STORAGE};
use futures::future;
use std::any::Any;
use std::fmt::Debug;
use stremio_derive::Model;
use tokio::runtime::current_thread::run;

#[test]
fn actionctx_logout() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/logout"
                && method == "POST"
                && body == "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    let profile = Profile {
        auth: Some(Auth {
            key: "auth_key".to_owned(),
            user: User {
                id: "user_id".to_owned(),
                email: "user_email".to_owned(),
                fb_id: None,
                avatar: None,
                last_modified: Env::now(),
                date_registered: Env::now(),
            },
        }),
        ..Default::default()
    };
    let library = LibBucket {
        uid: profile.uid(),
        ..Default::default()
    };
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string::<(UID, Vec<LibItem>)>(&(profile.uid(), vec![])).unwrap(),
    );
    STORAGE.write().unwrap().insert(
        LIBRARY_STORAGE_KEY.to_owned(),
        serde_json::to_string::<(UID, Vec<LibItem>)>(&(profile.uid(), vec![])).unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                library,
                ..Default::default()
            },
        },
        1000,
    );
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::Logout))));
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile,
        Default::default(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.library,
        Default::default(),
        "library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data).unwrap() == Default::default()
            }),
        "profile updated successfully in storage"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<(UID, Vec<LibItem>)>(&data).unwrap() == Default::default()
            }),
        "recent library updated successfully in storage"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<(UID, Vec<LibItem>)>(&data).unwrap() == Default::default()
            }),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/logout".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}".to_owned(),
            ..Default::default()
        },
        "Logout request has been sent"
    );
}
