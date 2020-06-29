use super::{Env, Request, REQUESTS, STORAGE};
use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{APIResult, Auth, SuccessResponse, True, User};
use crate::types::profile::{Profile, UID};
use crate::types::{LibBucket, LibItem};
use futures::future;
use serde::de::DeserializeOwned;
use std::any::Any;
use std::fmt::Debug;
use stremio_derive::Model;
use tokio::runtime::current_thread::run;

#[test]
fn actionctx_logout() {
    impl Env {
        pub fn unit_test_fetch<T: 'static + DeserializeOwned>(request: Request) -> EnvFuture<T> {
            match request {
                Request {
                    url, method, body, ..
                } if url == "https://api.strem.io/api/logout"
                    && method == "POST"
                    && body == "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}" =>
                {
                    let result: Box<dyn Any> = Box::new(APIResult::Ok {
                        result: SuccessResponse { success: True {} },
                    });
                    Box::new(future::ok(*result.downcast::<T>().unwrap()))
                }
                _ => panic!("Unhandled fetch request: {:#?}", request),
            }
        }
    }
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
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
    assert!(
        runtime.app.read().unwrap().ctx.profile.auth.is_none(),
        "profile updated successfully in memory"
    );
    assert!(
        runtime.app.read().unwrap().ctx.library.uid.is_none()
            && runtime.app.read().unwrap().ctx.library.items.is_empty(),
        "library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(true, |data| {
                serde_json::from_str::<Profile>(&data)
                    .unwrap()
                    .auth
                    .is_none()
            }),
        "profile updated successfully in storage"
    );
    // TODO library updated successfully in storage
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been send"
    );
    assert!(
        match REQUESTS.read().unwrap().get(0).unwrap() {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/logout"
                && method == "POST"
                && body == "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}" =>
                true,
            _ => false,
        },
        "Logout request has been send"
    );
}
