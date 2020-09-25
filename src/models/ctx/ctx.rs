use crate::constants::LIBRARY_COLLECTION_NAME;
use crate::models::ctx::{fetch_api, update_library, update_profile};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, Effects, Env, Update};
use crate::types::api::{
    APIRequest, AuthRequest, AuthResponse, CollectionResponse, DatastoreCommand, DatastoreRequest,
    SuccessResponse,
};
use crate::types::library::LibBucket;
use crate::types::profile::{Auth, Profile};
use derivative::Derivative;
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::marker::PhantomData;

#[derive(Derivative, PartialEq)]
#[derivative(Default)]
pub enum CtxStatus {
    Loading(AuthRequest),
    #[derivative(Default)]
    Ready,
}

#[derive(Derivative, Serialize)]
#[derivative(Default)]
pub struct Ctx<E: Env> {
    pub profile: Profile,
    // TODO StreamsBucket
    // TODO SubtitlesBucket
    // TODO SearchesBucket
    #[serde(serialize_with = "serialize_lib_bucket")]
    pub library: LibBucket,
    #[serde(skip)]
    pub status: CtxStatus,
    #[serde(skip)]
    pub env: PhantomData<E>,
}

impl<E: Env> Ctx<E> {
    pub fn new(profile: Profile, library: LibBucket) -> Self {
        Ctx {
            profile,
            library,
            status: CtxStatus::Ready,
            env: PhantomData,
        }
    }
}

impl<E: Env + 'static> Update for Ctx<E> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Ctx(ActionCtx::Authenticate(auth_request))) => {
                self.status = CtxStatus::Loading(auth_request.to_owned());
                Effects::one(authenticate::<E>(auth_request)).unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
                let uid = self.profile.uid();
                let session_effects = match &self.profile.auth {
                    Some(auth) => Effects::one(delete_session::<E>(&auth.key)).unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects = update_library::<E>(
                    &mut self.library,
                    self.profile.auth.as_ref().map(|auth| &auth.key),
                    &self.status,
                    &msg,
                );
                self.status = CtxStatus::Ready;
                Effects::msg(Msg::Event(Event::UserLoggedOut { uid }))
                    .unchanged()
                    .join(session_effects)
                    .join(profile_effects)
                    .join(library_effects)
            }
            Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => {
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects = update_library::<E>(
                    &mut self.library,
                    self.profile.auth.as_ref().map(|auth| &auth.key),
                    &self.status,
                    msg,
                );
                let ctx_effects = match &self.status {
                    CtxStatus::Loading(loading_auth_request)
                        if loading_auth_request == auth_request =>
                    {
                        self.status = CtxStatus::Ready;
                        match result {
                            Ok(_) => Effects::msg(Msg::Event(Event::UserAuthenticated {
                                auth_request: auth_request.to_owned(),
                            }))
                            .unchanged(),
                            Err(error) => Effects::msg(Msg::Event(Event::Error {
                                error: error.to_owned(),
                                source: Box::new(Event::UserAuthenticated {
                                    auth_request: auth_request.to_owned(),
                                }),
                            }))
                            .unchanged(),
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                profile_effects.join(library_effects).join(ctx_effects)
            }
            _ => {
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, &msg);
                let library_effects = update_library::<E>(
                    &mut self.library,
                    self.profile.auth.as_ref().map(|auth| &auth.key),
                    &self.status,
                    &msg,
                );
                profile_effects.join(library_effects)
            }
        }
    }
}

fn authenticate<E: Env + 'static>(auth_request: &AuthRequest) -> Effect {
    fetch_api::<E, _, _>(&APIRequest::Auth(auth_request.to_owned()))
        .map_ok(|AuthResponse { key, user }| Auth { key, user })
        .and_then(|auth| {
            future::try_join(
                fetch_api::<E, _, _>(&APIRequest::AddonCollectionGet {
                    auth_key: auth.key.to_owned(),
                    update: true,
                })
                .map_ok(|CollectionResponse { addons, .. }| addons),
                fetch_api::<E, _, _>(&DatastoreRequest {
                    auth_key: auth.key.to_owned(),
                    collection: LIBRARY_COLLECTION_NAME.to_owned(),
                    command: DatastoreCommand::Get {
                        ids: vec![],
                        all: true,
                    },
                }),
            )
            .map_ok(move |(addons, lib_items)| (auth, addons, lib_items))
        })
        .map(enclose!((auth_request) move |result| {
            Msg::Internal(Internal::CtxAuthResult(auth_request, result))
        }))
        .boxed_local()
        .into()
}

fn delete_session<E: Env + 'static>(auth_key: &str) -> Effect {
    fetch_api::<E, _, SuccessResponse>(&APIRequest::Logout {
        auth_key: auth_key.to_owned(),
    })
    .map(enclose!((auth_key.to_owned() => auth_key) move |result|
        match result {
            Ok(_) => Msg::Event(Event::SessionDeleted { auth_key }),
            Err(error) => Msg::Event(Event::Error {
                error,
                source: Box::new(Event::SessionDeleted { auth_key }),
            }),
        }
    ))
    .boxed_local()
    .into()
}

fn serialize_lib_bucket<S>(lib_bucket: &LibBucket, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct LibItemProjection {
        pub removed: bool,
        pub temp: bool,
    }
    let mut map = serializer.serialize_map(Some(lib_bucket.items.len()))?;
    for lib_item in lib_bucket.items.values() {
        map.serialize_entry(
            &lib_item.id,
            &LibItemProjection {
                removed: lib_item.removed,
                temp: lib_item.temp,
            },
        )?;
    }
    map.end()
}
