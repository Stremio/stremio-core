use crate::constants::{LIBRARY_COLLECTION_NAME, URI_COMPONENT_ENCODE_SET};
use crate::models::{
    common::{descriptor_update, eq_update, DescriptorAction, DescriptorLoadable, Loadable},
    ctx::{update_library, update_profile, CtxError, OtherError},
    local_search::LocalSearch,
};
use crate::runtime::{
    msg::{Action, ActionCtx, Event, Internal, Msg},
    Effect, EffectFuture, Effects, Env, EnvFutureExt, Update,
};
use crate::types::{
    api::{
        fetch_api, APIRequest, APIResult, AuthRequest, AuthResponse, CollectionResponse,
        DatastoreCommand, DatastoreRequest, LibraryItemsResponse, SuccessResponse,
    },
    library::LibraryBucket,
    profile::{Auth, AuthKey, Profile},
};
use derivative::Derivative;
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};
use percent_encoding::utf8_percent_encode;
use serde::Serialize;
use url::Url;

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub enum CtxStatus {
    Loading(AuthRequest),
    Ready,
}

#[derive(Derivative, Serialize, Clone, Debug)]
#[derivative(Default)]
pub struct Ctx {
    pub profile: Profile,
    // TODO StreamsBucket
    // TODO SubtitlesBucket
    // TODO SearchesBucket
    #[serde(skip)]
    pub local_search: LocalSearch,
    #[serde(skip)]
    pub library: LibraryBucket,
    #[serde(skip)]
    #[derivative(Default(value = "CtxStatus::Ready"))]
    pub status: CtxStatus,
    #[serde(skip)]
    pub trakt_addon: Option<DescriptorLoadable>,
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
                let trakt_addon_effects = eq_update(&mut self.trakt_addon, None);
                Effects::one(authenticate::<E>(auth_request))
                    .unchanged()
                    .join(trakt_addon_effects)
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) | Msg::Internal(Internal::Logout) => {
                let uid = self.profile.uid();
                let session_effects = match self.profile.auth_key() {
                    Some(auth_key) => Effects::one(delete_session::<E>(auth_key)).unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
                let trakt_addon_effects = eq_update(&mut self.trakt_addon, None);
                self.status = CtxStatus::Ready;
                Effects::msg(Msg::Event(Event::UserLoggedOut { uid }))
                    .unchanged()
                    .join(session_effects)
                    .join(profile_effects)
                    .join(library_effects)
                    .join(trakt_addon_effects)
            }
            Msg::Action(Action::Ctx(ActionCtx::InstallTraktAddon)) => {
                let uid = self.profile.uid();
                match uid {
                    Some(uid) => descriptor_update::<E>(
                        &mut self.trakt_addon,
                        DescriptorAction::DescriptorRequested {
                            transport_url: &Url::parse(&format!(
                                "https://www.strem.io/trakt/addon/{}/manifest.json",
                                utf8_percent_encode(&uid, URI_COMPONENT_ENCODE_SET)
                            ))
                            .expect("Failed to parse trakt addon transport url"),
                        },
                    ),
                    _ => Effects::msg(Msg::Event(Event::Error {
                        error: CtxError::from(OtherError::UserNotLoggedIn),
                        source: Box::new(Event::TraktAddonFetched { uid }),
                    }))
                    .unchanged(),
                }
            }
            Msg::Internal(Internal::ManifestRequestResult(transport_url, result)) => {
                let trakt_addon_effects = descriptor_update::<E>(
                    &mut self.trakt_addon,
                    DescriptorAction::ManifestRequestResult {
                        transport_url,
                        result,
                    },
                );
                let trakt_addon_events_effects = match &self.trakt_addon {
                    Some(DescriptorLoadable {
                        content: Loadable::Ready(addon),
                        ..
                    }) if trakt_addon_effects.has_changed => Effects::msgs(vec![
                        Msg::Event(Event::TraktAddonFetched {
                            uid: self.profile.uid(),
                        }),
                        Msg::Internal(Internal::InstallAddon(addon.to_owned())),
                    ])
                    .unchanged(),
                    Some(DescriptorLoadable {
                        content: Loadable::Err(error),
                        ..
                    }) if trakt_addon_effects.has_changed => {
                        Effects::msg(Msg::Event(Event::Error {
                            error: CtxError::Env(error.to_owned()),
                            source: Box::new(Event::TraktAddonFetched {
                                uid: self.profile.uid(),
                            }),
                        }))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                };
                self.trakt_addon = None;
                trakt_addon_effects.join(trakt_addon_events_effects)
            }
            Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => {
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
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
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
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
                    fetch_api::<E, _, _, LibraryItemsResponse>(&DatastoreRequest {
                        auth_key: auth.key.to_owned(),
                        collection: LIBRARY_COLLECTION_NAME.to_owned(),
                        command: DatastoreCommand::Get {
                            ids: vec![],
                            all: true,
                        },
                    })
                    .map_err(CtxError::from)
                    .and_then(|result| match result {
                        APIResult::Ok { result } => future::ok(result.0),
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
