use crate::{
    constants::LIBRARY_COLLECTION_NAME,
    models::{
        common::{DescriptorLoadable, Loadable, ResourceLoadable},
        ctx::{
            update_events, update_library, update_notifications, update_profile,
            update_search_history, update_streaming_server_urls, update_streams,
            update_trakt_addon, CtxError, OtherError,
        },
    },
    runtime::{
        msg::{Action, ActionCtx, CtxAuthResponse, Event, Internal, Msg},
        Effect, EffectFuture, Effects, Env, EnvFutureExt, Update,
    },
    types::{
        api::{
            fetch_api, APIRequest, APIResult, AuthRequest, AuthResponse, CollectionResponse,
            DatastoreCommand, DatastoreRequest, LibraryItemsResponse, SuccessResponse,
        },
        events::{DismissedEventsBucket, Events},
        library::LibraryBucket,
        notifications::NotificationsBucket,
        profile::{Auth, AuthKey, Profile},
        resource::MetaItem,
        search_history::SearchHistoryBucket,
        server_urls::ServerUrlsBucket,
        streams::StreamsBucket,
    },
};

#[cfg(test)]
use derivative::Derivative;
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};
use serde::Serialize;

use tracing::{error, trace};

#[derive(Default, PartialEq, Eq, Serialize, Clone, Debug)]
pub enum CtxStatus {
    Loading(AuthRequest),
    #[default]
    Ready,
}

#[derive(Serialize, Clone, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct Ctx {
    pub profile: Profile,
    // TODO SubtitlesBucket
    // TODO SearchesBucket
    #[serde(skip)]
    pub library: LibraryBucket,
    pub notifications: NotificationsBucket,
    #[serde(skip)]
    pub streams: StreamsBucket,
    #[serde(skip)]
    pub streaming_server_urls: ServerUrlsBucket,
    #[serde(skip)]
    pub search_history: SearchHistoryBucket,
    #[serde(skip)]
    pub dismissed_events: DismissedEventsBucket,
    #[serde(skip)]
    #[cfg_attr(test, derivative(Default(value = "CtxStatus::Ready")))]
    pub status: CtxStatus,
    #[serde(skip)]
    /// Used only for loading the Descriptor and then the descriptor will be discarded
    pub trakt_addon: Option<DescriptorLoadable>,
    #[serde(skip)]
    /// The catalogs response from all addons that support the `lastVideosIds`
    /// ([`LAST_VIDEOS_IDS_EXTRA_PROP`]) resource
    ///
    /// [`LAST_VIDEOS_IDS_EXTRA_PROP`]: static@crate::constants::LAST_VIDEOS_IDS_EXTRA_PROP
    pub notification_catalogs: Vec<ResourceLoadable<Vec<MetaItem>>>,

    pub events: Events,
}

impl Ctx {
    pub fn new(
        profile: Profile,
        library: LibraryBucket,
        streams: StreamsBucket,
        streaming_server_urls: ServerUrlsBucket,
        notifications: NotificationsBucket,

        search_history: SearchHistoryBucket,
        dismissed_events: DismissedEventsBucket,
    ) -> Self {
        Self {
            profile,
            library,
            streams,
            streaming_server_urls,
            search_history,
            dismissed_events,
            notifications,
            trakt_addon: None,
            notification_catalogs: vec![],
            status: CtxStatus::Ready,
            events: Events {
                modal: Loadable::Loading,
                notification: Loadable::Loading,
            },
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
                Effects::msg(Msg::Internal(Internal::Logout(false))).unchanged()
            }
            Msg::Internal(Internal::Logout(deleted)) => {
                let uid = self.profile.uid();
                let session_effects = match self.profile.auth_key() {
                    Some(auth_key) if !deleted => {
                        Effects::one(delete_session::<E>(auth_key)).unchanged()
                    }
                    _ => Effects::none().unchanged(),
                };
                let profile_effects =
                    update_profile::<E>(&mut self.profile, &mut self.streams, &self.status, msg);
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
                let streams_effects = update_streams::<E>(&mut self.streams, &self.status, msg);
                let server_urls_effects = update_streaming_server_urls::<E>(
                    &mut self.streaming_server_urls,
                    &self.status,
                    msg,
                );
                let search_history_effects =
                    update_search_history::<E>(&mut self.search_history, &self.status, msg);
                let events_effects =
                    update_events::<E>(&mut self.events, &mut self.dismissed_events, msg);
                let trakt_addon_effects = update_trakt_addon::<E>(
                    &mut self.trakt_addon,
                    &self.profile,
                    &self.status,
                    msg,
                );
                let notifications_effects = update_notifications::<E>(
                    &mut self.notifications,
                    &mut self.notification_catalogs,
                    &self.profile,
                    &self.library,
                    &self.status,
                    msg,
                );
                self.status = CtxStatus::Ready;
                Effects::msg(Msg::Event(Event::UserLoggedOut { uid }))
                    .unchanged()
                    .join(session_effects)
                    .join(profile_effects)
                    .join(library_effects)
                    .join(streams_effects)
                    .join(server_urls_effects)
                    .join(search_history_effects)
                    .join(events_effects)
                    .join(trakt_addon_effects)
                    .join(notifications_effects)
            }
            Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => {
                let profile_effects =
                    update_profile::<E>(&mut self.profile, &mut self.streams, &self.status, msg);
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
                let trakt_addon_effects = update_trakt_addon::<E>(
                    &mut self.trakt_addon,
                    &self.profile,
                    &self.status,
                    msg,
                );
                let notifications_effects = update_notifications::<E>(
                    &mut self.notifications,
                    &mut self.notification_catalogs,
                    &self.profile,
                    &self.library,
                    &self.status,
                    msg,
                );
                let streams_effects = update_streams::<E>(&mut self.streams, &self.status, msg);
                let server_urls_effects = update_streaming_server_urls::<E>(
                    &mut self.streaming_server_urls,
                    &self.status,
                    msg,
                );
                let search_history_effects =
                    update_search_history::<E>(&mut self.search_history, &self.status, msg);
                let events_effects =
                    update_events::<E>(&mut self.events, &mut self.dismissed_events, msg);
                let ctx_effects = match &self.status {
                    CtxStatus::Loading(loading_auth_request)
                        if loading_auth_request == auth_request =>
                    {
                        self.status = CtxStatus::Ready;
                        match result {
                            Ok(ctx_auth) => {
                                let authentication_effects =
                                    Effects::msg(Msg::Event(Event::UserAuthenticated {
                                        auth_request: auth_request.to_owned(),
                                    }))
                                    .unchanged();

                                let addons_locked_event = Event::UserAddonsLocked {
                                    addons_locked: ctx_auth.addons_result.is_err(),
                                };

                                let addons_effects = match ctx_auth.addons_result.as_ref() {
                                    Ok(_addons) => {
                                        Effects::msg(Msg::Event(addons_locked_event)).unchanged()
                                    }
                                    Err(_err) => Effects::msg(Msg::Event(Event::Error {
                                        error: CtxError::Other(OtherError::UserAddonsAreLocked),
                                        source: Box::new(addons_locked_event),
                                    }))
                                    .unchanged(),
                                };

                                let library_missing_event = Event::UserLibraryMissing {
                                    library_missing: ctx_auth.library_items_result.is_err(),
                                };

                                let library_missing_effects = match ctx_auth
                                    .library_items_result
                                    .as_ref()
                                {
                                    Ok(_library) => {
                                        Effects::msg(Msg::Event(library_missing_event)).unchanged()
                                    }
                                    Err(_err) => Effects::msg(Msg::Event(Event::Error {
                                        error: CtxError::Other(OtherError::UserLibraryIsMissing),
                                        source: Box::new(library_missing_event),
                                    }))
                                    .unchanged(),
                                };

                                authentication_effects
                                    .join(addons_effects)
                                    .join(library_missing_effects)
                            }
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
                profile_effects
                    .join(library_effects)
                    .join(streams_effects)
                    .join(server_urls_effects)
                    .join(trakt_addon_effects)
                    .join(notifications_effects)
                    .join(search_history_effects)
                    .join(events_effects)
                    .join(ctx_effects)
            }
            _ => {
                let profile_effects =
                    update_profile::<E>(&mut self.profile, &mut self.streams, &self.status, msg);
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
                let streams_effects = update_streams::<E>(&mut self.streams, &self.status, msg);
                let server_urls_effects = update_streaming_server_urls::<E>(
                    &mut self.streaming_server_urls,
                    &self.status,
                    msg,
                );
                let trakt_addon_effects = update_trakt_addon::<E>(
                    &mut self.trakt_addon,
                    &self.profile,
                    &self.status,
                    msg,
                );
                let notifications_effects = update_notifications::<E>(
                    &mut self.notifications,
                    &mut self.notification_catalogs,
                    &self.profile,
                    &self.library,
                    &self.status,
                    msg,
                );
                let search_history_effects =
                    update_search_history::<E>(&mut self.search_history, &self.status, msg);
                let events_effects =
                    update_events::<E>(&mut self.events, &mut self.dismissed_events, msg);
                profile_effects
                    .join(library_effects)
                    .join(streams_effects)
                    .join(server_urls_effects)
                    .join(trakt_addon_effects)
                    .join(notifications_effects)
                    .join(search_history_effects)
                    .join(events_effects)
            }
        }
    }
}

fn authenticate<E: Env + 'static>(auth_request: &AuthRequest) -> Effect {
    let auth_api = APIRequest::Auth(auth_request.clone());

    EffectFuture::Concurrent(
        async {
            E::flush_analytics().await;

            // return an error only if the auth request fails
            let auth = fetch_api::<E, _, _, _>(&auth_api)
                .inspect(move |result| trace!(?result, ?auth_api, "Auth request"))
                .await
                .map_err(CtxError::from)
                .and_then(|result| match result {
                    APIResult::Ok(result) => Ok(result),
                    APIResult::Err(error) => Err(CtxError::from(error)),
                })
                .map(|AuthResponse { key, user }| Auth { key, user })?;

            let addon_collection_fut = async {
                let request = APIRequest::AddonCollectionGet {
                    auth_key: auth.key.to_owned(),
                    update: true,
                };
                fetch_api::<E, _, _, _>(&request)
                    .inspect(move |result| {
                        trace!(?result, ?request, "Get user's Addon Collection request")
                    })
                    .await
                    .map_err(CtxError::from)
                    .and_then(|result: APIResult<CollectionResponse>| match result {
                        APIResult::Ok(result) => Ok(result),
                        APIResult::Err(error) => Err(CtxError::from(error)),
                    })
                    .map(|CollectionResponse { addons, .. }| addons)
            };

            let datastore_library_fut = async {
                let request = DatastoreRequest {
                    auth_key: auth.key.to_owned(),
                    collection: LIBRARY_COLLECTION_NAME.to_owned(),
                    command: DatastoreCommand::Get {
                        ids: vec![],
                        all: true,
                    },
                };

                fetch_api::<E, _, _, LibraryItemsResponse>(&request)
                    .inspect(move |result| {
                        trace!(?result, ?request, "Get user's Addon Collection request")
                    })
                    .await
                    .map_err(CtxError::from)
                    .and_then(|result| match result {
                        APIResult::Ok(result) => Ok(result.0),
                        APIResult::Err(error) => Err(CtxError::from(error)),
                    })
            };

            let (addon_collection_result, datastore_library_result) =
                future::join(addon_collection_fut, datastore_library_fut).await;

            if let Err(error) = addon_collection_result.as_ref() {
                error!("Failed to fetch Addon collection from API: {error:?}");
            }
            if let Err(error) = datastore_library_result.as_ref() {
                error!("Failed to fetch LibraryItems for user from API: {error:?}");
            }

            Ok(CtxAuthResponse {
                auth,
                addons_result: addon_collection_result,
                library_items_result: datastore_library_result,
            })
        }
        .map(enclose!((auth_request) move |result| {
            Msg::Internal(Internal::CtxAuthResult(auth_request, result))
        }))
        .boxed_env(),
    )
    .into()
}

fn delete_session<E: Env + 'static>(auth_key: &AuthKey) -> Effect {
    let request = APIRequest::Logout {
        auth_key: auth_key.clone(),
    };

    EffectFuture::Concurrent(
        E::flush_analytics()
            .then(|_| {
                fetch_api::<E, _, _, SuccessResponse>(&request)
                    .inspect(move |result| trace!(?result, ?request, "Logout request"))
            })
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
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
