use crate::constants::LIBRARY_COLLECTION_NAME;
use crate::models::ctx::{update_library, update_profile, CtxError};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt, Update};
use crate::types::api::{
    fetch_api, APIRequest, APIResult, AuthRequest, AuthResponse, CollectionResponse,
    DatastoreCommand, DatastoreRequest, SuccessResponse,
};
use crate::types::library::LibraryBucket;
use crate::types::profile::{Auth, AuthKey, Profile};
use derivative::Derivative;
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};
use serde::Serialize;

#[derive(PartialEq, Serialize)]
#[cfg_attr(debug_assertions, derive(Clone))]
pub enum CtxStatus {
    Loading(AuthRequest),
    Ready,
}

#[derive(Derivative, Serialize)]
#[cfg_attr(debug_assertions, derive(Clone))]
#[derivative(Default)]
pub struct Ctx {
    pub profile: Profile,
    // TODO StreamsBucket
    // TODO SubtitlesBucket
    // TODO SearchesBucket
    #[serde(skip)]
    pub library: LibraryBucket,
    #[serde(skip)]
    #[derivative(Default(value = "CtxStatus::Ready"))]
    pub status: CtxStatus,
}

impl Ctx {
    pub fn new(profile: Profile, library: LibraryBucket) -> Self {
        Self {
            profile,
            library,
            ..Self::default()
        }
    }
}

impl<E: Env + 'static> Update<E> for Ctx {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Ctx(ActionCtx::Authenticate(auth_request))) => {
                self.status = CtxStatus::Loading(auth_request.to_owned());
                Effects::one(authenticate::<E>(auth_request)).unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
                let uid = self.profile.uid();
                let session_effects = match self.profile.auth_key() {
                    Some(auth_key) => Effects::one(delete_session::<E>(auth_key)).unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects = update_library::<E>(
                    &mut self.library,
                    self.profile.auth_key(),
                    &self.status,
                    msg,
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
                    self.profile.auth_key(),
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
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects = update_library::<E>(
                    &mut self.library,
                    self.profile.auth_key(),
                    &self.status,
                    msg,
                );
                profile_effects.join(library_effects)
            }
        }
    }
}

fn authenticate<E: Env + 'static>(auth_request: &AuthRequest) -> Effect {
    EffectFuture::Concurrent(
        E::flush_analytics()
            .then(enclose!((auth_request) move |_| {
                fetch_api::<E, _, _, _>(&APIRequest::Auth(auth_request))
            }))
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok { result } => future::ok(result),
                APIResult::Err { error } => future::err(CtxError::from(error)),
            })
            .map_ok(|AuthResponse { key, user }| Auth { key, user })
            .and_then(|auth| {
                future::try_join(
                    fetch_api::<E, _, _, _>(&APIRequest::AddonCollectionGet {
                        auth_key: auth.key.to_owned(),
                        update: true,
                    })
                    .map_err(CtxError::from)
                    .and_then(|result| match result {
                        APIResult::Ok { result } => future::ok(result),
                        APIResult::Err { error } => future::err(CtxError::from(error)),
                    })
                    .map_ok(|CollectionResponse { addons, .. }| addons),
                    fetch_api::<E, _, _, _>(&DatastoreRequest {
                        auth_key: auth.key.to_owned(),
                        collection: LIBRARY_COLLECTION_NAME.to_owned(),
                        command: DatastoreCommand::Get {
                            ids: vec![],
                            all: true,
                        },
                    })
                    .map_err(CtxError::from)
                    .and_then(|result| match result {
                        APIResult::Ok { result } => future::ok(result),
                        APIResult::Err { error } => future::err(CtxError::from(error)),
                    }),
                )
                .map_ok(move |(addons, library_items)| (auth, addons, library_items))
            })
            .map(enclose!((auth_request) move |result| {
                Msg::Internal(Internal::CtxAuthResult(auth_request, result))
            }))
            .boxed_env(),
    )
    .into()
}

fn delete_session<E: Env + 'static>(auth_key: &AuthKey) -> Effect {
    EffectFuture::Concurrent(
        E::flush_analytics()
            .then(enclose!((auth_key) move |_| {
                fetch_api::<E, _, _, SuccessResponse>(&APIRequest::Logout { auth_key })
            }))
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok { result } => future::ok(result),
                APIResult::Err { error } => future::err(CtxError::from(error)),
            })
            .map(enclose!((auth_key) move |result| match result {
                Ok(_) => Msg::Event(Event::SessionDeleted { auth_key }),
                Err(error) => Msg::Event(Event::Error {
                    error,
                    source: Box::new(Event::SessionDeleted { auth_key }),
                }),
            }))
            .boxed_env(),
    )
    .into()
}
