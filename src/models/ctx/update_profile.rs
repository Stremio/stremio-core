use std::collections::HashSet;

use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};

use crate::constants::{OFFICIAL_ADDONS, PROFILE_STORAGE_KEY};
use crate::models::ctx::{CtxError, CtxStatus, OtherError};
use crate::runtime::msg::{Action, ActionCtx, CtxAuthResponse, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt};
use crate::types::addon::Descriptor;
use crate::types::api::{
    fetch_api, APIError, APIRequest, APIResult, CollectionResponse, SuccessResponse,
};
use crate::types::profile::{Auth, AuthKey, Password, Profile, Settings, User};
use crate::types::streams::StreamsBucket;

pub fn update_profile<E: Env + 'static>(
    profile: &mut Profile,
    streams: &mut StreamsBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Internal(Internal::Logout(_)) => {
            let next_profile = Profile::default();
            if *profile != next_profile {
                *profile = next_profile;
                Effects::msg(Msg::Internal(Internal::ProfileChanged))
            } else {
                Effects::none().unchanged()
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::DeleteAccount(password))) => match profile.auth_key() {
            Some(auth_key) => Effects::one(delete_account::<E>(auth_key, password)).unchanged(),
            _ => Effects::msg(Msg::Event(Event::Error {
                error: CtxError::from(OtherError::UserNotLoggedIn),
                source: Box::new(Event::UserAccountDeleted { uid: profile.uid() }),
            }))
            .unchanged(),
        },
        Msg::Action(Action::Ctx(ActionCtx::PushUserToAPI)) => match &profile.auth {
            Some(Auth { key, user }) => {
                Effects::one(push_user_to_api::<E>(user.to_owned(), key)).unchanged()
            }
            _ => Effects::msg(Msg::Event(Event::Error {
                error: CtxError::from(OtherError::UserNotLoggedIn),
                source: Box::new(Event::UserPushedToAPI { uid: profile.uid() }),
            }))
            .unchanged(),
        },
        Msg::Action(Action::Ctx(ActionCtx::PullUserFromAPI)) => match profile.auth_key() {
            Some(auth_key) => Effects::one(pull_user_from_api::<E>(auth_key)).unchanged(),
            _ => Effects::msg(Msg::Event(Event::Error {
                error: CtxError::from(OtherError::UserNotLoggedIn),
                source: Box::new(Event::UserPulledFromAPI { uid: profile.uid() }),
            }))
            .unchanged(),
        },
        Msg::Action(Action::Ctx(ActionCtx::PushAddonsToAPI)) => match profile.auth_key() {
            Some(auth_key) => {
                Effects::one(push_addons_to_api::<E>(profile.addons.to_owned(), auth_key))
                    .unchanged()
            }
            _ => Effects::msg(Msg::Event(Event::Error {
                error: CtxError::from(OtherError::UserNotLoggedIn),
                source: Box::new(Event::AddonsPushedToAPI {
                    transport_urls: profile
                        .addons
                        .iter()
                        .map(|addon| &addon.transport_url)
                        .cloned()
                        .collect(),
                }),
            }))
            .unchanged(),
        },
        Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI)) => match profile.auth_key() {
            Some(auth_key) => Effects::one(pull_addons_from_api::<E>(auth_key)).unchanged(),
            _ => {
                let next_addons = profile
                    .addons
                    .iter()
                    .map(|profile_addon| {
                        OFFICIAL_ADDONS
                            .iter()
                            .find(|Descriptor { manifest, .. }| {
                                manifest.id == profile_addon.manifest.id
                                    && manifest.version > profile_addon.manifest.version
                            })
                            .map(|official_addon| Descriptor {
                                transport_url: official_addon.transport_url.to_owned(),
                                manifest: official_addon.manifest.to_owned(),
                                flags: profile_addon.flags.to_owned(),
                            })
                            .unwrap_or_else(|| profile_addon.to_owned())
                    })
                    .collect::<Vec<_>>();
                let prev_transport_urls = profile
                    .addons
                    .iter()
                    .map(|addon| &addon.transport_url)
                    .cloned()
                    .collect::<HashSet<_>>();
                let next_transport_urls = next_addons
                    .iter()
                    .map(|addon| &addon.transport_url)
                    .cloned()
                    .collect::<HashSet<_>>();
                let added_transport_urls = &next_transport_urls - &prev_transport_urls;
                let removed_transport_urls = &prev_transport_urls - &next_transport_urls;
                let transport_urls = added_transport_urls
                    .into_iter()
                    .chain(removed_transport_urls)
                    .collect();
                if profile.addons != next_addons {
                    profile.addons = next_addons;
                    Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { transport_urls }))
                        .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { transport_urls }))
                        .unchanged()
                }
            }
        },
        Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon))) => {
            Effects::msg(Msg::Internal(Internal::InstallAddon(addon.to_owned()))).unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(addon))) => {
            Effects::msg(Msg::Internal(Internal::UninstallAddon(addon.to_owned()))).unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::UpgradeAddon(addon))) => {
            if profile.addons_locked {
                return addon_upgrade_error_effects(addon, OtherError::UserAddonsAreLocked);
            }

            if profile.addons.contains(addon) {
                return addon_upgrade_error_effects(addon, OtherError::AddonAlreadyInstalled);
            }
            if addon.manifest.behavior_hints.configuration_required {
                return addon_upgrade_error_effects(addon, OtherError::AddonConfigurationRequired);
            }
            let addon_position = match profile
                .addons
                .iter()
                .map(|addon| &addon.transport_url)
                .position(|transport_url| *transport_url == addon.transport_url)
            {
                Some(addon_position) => addon_position,
                None => return addon_upgrade_error_effects(addon, OtherError::AddonNotInstalled),
            };
            if addon.flags.protected || profile.addons[addon_position].flags.protected {
                return addon_upgrade_error_effects(addon, OtherError::AddonIsProtected);
            }
            addon.clone_into(&mut profile.addons[addon_position]);
            let push_to_api_effects = match profile.auth_key() {
                Some(auth_key) => {
                    Effects::one(push_addons_to_api::<E>(profile.addons.to_owned(), auth_key))
                        .unchanged()
                }
                _ => Effects::none().unchanged(),
            };
            Effects::msg(Msg::Event(Event::AddonUpgraded {
                transport_url: addon.transport_url.to_owned(),
                id: addon.manifest.id.to_owned(),
            }))
            .join(push_to_api_effects)
            .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
        }
        Msg::Internal(Internal::UninstallAddon(addon)) => {
            if profile.addons_locked {
                return addon_uninstall_error_effects(addon, OtherError::UserAddonsAreLocked);
            }

            let addon_position = profile
                .addons
                .iter()
                .map(|addon| &addon.transport_url)
                .position(|transport_url| *transport_url == addon.transport_url);
            if let Some(addon_position) = addon_position {
                if !profile.addons[addon_position].flags.protected && !addon.flags.protected {
                    profile.addons.remove(addon_position);

                    // Remove stream related to this addon from the streams bucket
                    streams
                        .items
                        .retain(|_key, item| item.stream_transport_url != addon.transport_url);

                    let push_to_api_effects = match profile.auth_key() {
                        Some(auth_key) => Effects::one(push_addons_to_api::<E>(
                            profile.addons.to_owned(),
                            auth_key,
                        ))
                        .unchanged(),
                        _ => Effects::none().unchanged(),
                    };
                    Effects::msg(Msg::Event(Event::AddonUninstalled {
                        transport_url: addon.transport_url.to_owned(),
                        id: addon.manifest.id.to_owned(),
                    }))
                    .join(push_to_api_effects)
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    addon_uninstall_error_effects(addon, OtherError::AddonIsProtected)
                }
            } else {
                addon_uninstall_error_effects(addon, OtherError::AddonNotInstalled)
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::LogoutTrakt)) => match &mut profile.auth {
            Some(Auth { user, key }) => {
                if user.trakt.is_some() {
                    user.trakt = None;
                    let push_to_api_effects =
                        Effects::one(push_user_to_api::<E>(user.to_owned(), key));

                    Effects::msg(Msg::Event(Event::TraktLoggedOut { uid: profile.uid() }))
                        .join(push_to_api_effects)
                        // first uninstall the trakt addon
                        .join(Effects::msg(Msg::Internal(Internal::UninstallTraktAddon)))
                        .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::msg(Msg::Event(Event::TraktLoggedOut { uid: profile.uid() }))
                        .unchanged()
                }
            }
            _ => Effects::msg(Msg::Event(Event::Error {
                error: CtxError::from(OtherError::UserNotLoggedIn),
                source: Box::new(Event::TraktLoggedOut { uid: profile.uid() }),
            }))
            .unchanged(),
        },
        Msg::Action(Action::Ctx(ActionCtx::UpdateSettings(settings))) => {
            if profile.settings != *settings {
                settings.clone_into(&mut profile.settings);
                Effects::msg(Msg::Event(Event::SettingsUpdated {
                    settings: settings.to_owned(),
                }))
                .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
            } else {
                Effects::msg(Msg::Event(Event::SettingsUpdated {
                    settings: settings.to_owned(),
                }))
                .unchanged()
            }
        }
        Msg::Internal(Internal::ProfileChanged) => {
            Effects::one(push_profile_to_storage::<E>(profile)).unchanged()
        }
        Msg::Internal(Internal::InstallAddon(addon)) => {
            if profile.addons_locked {
                return addon_install_error_effects(addon, OtherError::UserAddonsAreLocked);
            }

            if !profile.addons.contains(addon) {
                if !addon.manifest.behavior_hints.configuration_required {
                    let addon_position = profile
                        .addons
                        .iter()
                        .map(|addon| &addon.transport_url)
                        .position(|transport_url| *transport_url == addon.transport_url);
                    if let Some(addon_position) = addon_position {
                        addon.clone_into(&mut profile.addons[addon_position]);
                    } else {
                        profile.addons.push(addon.to_owned());
                    };
                    let push_to_api_effects = match profile.auth_key() {
                        Some(auth_key) => Effects::one(push_addons_to_api::<E>(
                            profile.addons.to_owned(),
                            auth_key,
                        ))
                        .unchanged(),
                        _ => Effects::none().unchanged(),
                    };
                    Effects::msg(Msg::Event(Event::AddonInstalled {
                        transport_url: addon.transport_url.to_owned(),
                        id: addon.manifest.id.to_owned(),
                    }))
                    .join(push_to_api_effects)
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    addon_install_error_effects(addon, OtherError::AddonConfigurationRequired)
                }
            } else {
                addon_install_error_effects(addon, OtherError::AddonAlreadyInstalled)
            }
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (
                CtxStatus::Loading(loading_auth_request),
                Ok(CtxAuthResponse {
                    auth,
                    addons_result,
                    ..
                }),
            ) if loading_auth_request == auth_request => {
                let next_profile = Profile {
                    auth: Some(auth.to_owned()),
                    addons: addons_result.to_owned().unwrap_or(OFFICIAL_ADDONS.clone()),
                    addons_locked: addons_result.is_err(),
                    settings: Settings::default(),
                };
                if *profile != next_profile {
                    *profile = next_profile;
                    Effects::msg(Msg::Internal(Internal::ProfileChanged))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::AddonsAPIResult(
            APIRequest::AddonCollectionGet { auth_key, .. },
            result,
        )) if profile.auth_key() == Some(auth_key) => {
            let profile_effects = match result {
                Ok(addons) => {
                    let prev_transport_urls = profile
                        .addons
                        .iter()
                        .map(|addon| &addon.transport_url)
                        .cloned()
                        .collect::<HashSet<_>>();
                    let next_transport_urls = addons
                        .iter()
                        .map(|addon| &addon.transport_url)
                        .cloned()
                        .collect::<HashSet<_>>();
                    let added_transport_urls = &next_transport_urls - &prev_transport_urls;
                    let removed_transport_urls = &prev_transport_urls - &next_transport_urls;
                    let transport_urls = added_transport_urls
                        .into_iter()
                        .chain(removed_transport_urls)
                        .collect();
                    let profile_changed_effects = if profile.addons != *addons {
                        addons.clone_into(&mut profile.addons);

                        Effects::msg(Msg::Internal(Internal::ProfileChanged))
                    } else {
                        Effects::none().unchanged()
                    };

                    Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { transport_urls }))
                        .join(profile_changed_effects)
                }
                Err(error) => Effects::msg(Msg::Event(Event::Error {
                    error: error.to_owned(),
                    source: Box::new(Event::AddonsPulledFromAPI {
                        transport_urls: Default::default(),
                    }),
                }))
                .unchanged(),
            };

            // on successful AddonsApi result, unlock the addons if they have been locked
            // on failed AddonsApi result, lock the addons
            profile.addons_locked = result.is_err();
            let addons_locked_event = Event::UserAddonsLocked {
                addons_locked: profile.addons_locked,
            };
            let addons_locked_effects = Effects::msg(Msg::Event(addons_locked_event)).unchanged();

            addons_locked_effects.join(profile_effects)
        }
        Msg::Internal(Internal::UserAPIResult(APIRequest::GetUser { auth_key }, result))
            if profile.auth_key() == Some(auth_key) =>
        {
            match result {
                Ok(user) => match &mut profile.auth {
                    Some(auth) if auth.user != *user => {
                        user.clone_into(&mut auth.user);
                        Effects::msg(Msg::Event(Event::UserPulledFromAPI { uid: profile.uid() }))
                            .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                    }
                    _ => Effects::msg(Msg::Event(Event::UserPulledFromAPI { uid: profile.uid() }))
                        .unchanged(),
                },
                Err(error) => {
                    let session_expired_effects = match error {
                        CtxError::API(APIError { code, .. }) if *code == 1 => {
                            Effects::msg(Msg::Internal(Internal::Logout(false))).unchanged()
                        }
                        _ => Effects::none().unchanged(),
                    };
                    Effects::msg(Msg::Event(Event::Error {
                        error: error.to_owned(),
                        source: Box::new(Event::UserPulledFromAPI { uid: profile.uid() }),
                    }))
                    .unchanged()
                    .join(session_expired_effects)
                }
            }
        }
        Msg::Internal(Internal::DeleteAccountAPIResult(
            APIRequest::DeleteAccount { auth_key, .. },
            result,
        )) if profile.auth_key() == Some(auth_key) => match result {
            Ok(_) => Effects::msg(Msg::Internal(Internal::Logout(true))).unchanged(),
            Err(error) => Effects::msg(Msg::Event(Event::Error {
                error: error.to_owned(),
                source: Box::new(Event::UserAccountDeleted { uid: profile.uid() }),
            }))
            .unchanged(),
        },
        _ => Effects::none().unchanged(),
    }
}

fn push_addons_to_api<E: Env + 'static>(addons: Vec<Descriptor>, auth_key: &AuthKey) -> Effect {
    let transport_urls = addons
        .iter()
        .map(|addon| &addon.transport_url)
        .cloned()
        .collect();
    let request = APIRequest::AddonCollectionSet {
        auth_key: auth_key.to_owned(),
        addons,
    };
    EffectFuture::Concurrent(
        fetch_api::<E, _, _, SuccessResponse>(&request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result| match result {
                Ok(_) => Msg::Event(Event::AddonsPushedToAPI { transport_urls }),
                Err(error) => Msg::Event(Event::Error {
                    error,
                    source: Box::new(Event::AddonsPushedToAPI { transport_urls }),
                }),
            })
            .boxed_env(),
    )
    .into()
}

fn pull_user_from_api<E: Env + 'static>(auth_key: &AuthKey) -> Effect {
    let request = APIRequest::GetUser {
        auth_key: auth_key.to_owned(),
    };
    EffectFuture::Concurrent(
        fetch_api::<E, _, _, _>(&request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::UserAPIResult(request, result)))
            .boxed_env(),
    )
    .into()
}

fn push_user_to_api<E: Env + 'static>(user: User, auth_key: &AuthKey) -> Effect {
    let uid = Some(user.id.to_owned());
    let request = APIRequest::SaveUser {
        auth_key: auth_key.to_owned(),
        user,
    };
    EffectFuture::Concurrent(
        fetch_api::<E, _, _, SuccessResponse>(&request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result| match result {
                Ok(_) => Msg::Event(Event::UserPushedToAPI { uid }),
                Err(error) => Msg::Event(Event::Error {
                    error,
                    source: Box::new(Event::UserPushedToAPI { uid }),
                }),
            })
            .boxed_env(),
    )
    .into()
}

fn pull_addons_from_api<E: Env + 'static>(auth_key: &AuthKey) -> Effect {
    let request = APIRequest::AddonCollectionGet {
        auth_key: auth_key.to_owned(),
        update: true,
    };
    EffectFuture::Concurrent(
        fetch_api::<E, _, _, _>(&request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map_ok(|CollectionResponse { addons, .. }| addons)
            .map(move |result| Msg::Internal(Internal::AddonsAPIResult(request, result)))
            .boxed_env(),
    )
    .into()
}

fn push_profile_to_storage<E: Env + 'static>(profile: &Profile) -> Effect {
    EffectFuture::Sequential(
        E::set_storage(PROFILE_STORAGE_KEY, Some(profile))
            .map(enclose!((profile.uid() => uid) move |result| match result {
                Ok(_) => Msg::Event(Event::ProfilePushedToStorage { uid }),
                Err(error) => Msg::Event(Event::Error {
                    error: CtxError::from(error),
                    source: Box::new(Event::ProfilePushedToStorage { uid }),
                })
            }))
            .boxed_env(),
    )
    .into()
}

fn delete_account<E: Env + 'static>(auth_key: &AuthKey, password: &Password) -> Effect {
    let request = APIRequest::DeleteAccount {
        auth_key: auth_key.to_owned(),
        password: password.to_owned(),
    };
    EffectFuture::Concurrent(
        fetch_api::<E, _, _, _>(&request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::DeleteAccountAPIResult(request, result)))
            .boxed_env(),
    )
    .into()
}

fn addon_upgrade_error_effects(addon: &Descriptor, error: OtherError) -> Effects {
    addon_action_error_effects(
        error,
        Event::AddonUpgraded {
            transport_url: addon.transport_url.to_owned(),
            id: addon.manifest.id.to_owned(),
        },
    )
}

fn addon_uninstall_error_effects(addon: &Descriptor, error: OtherError) -> Effects {
    addon_action_error_effects(
        error,
        Event::AddonUninstalled {
            transport_url: addon.transport_url.to_owned(),
            id: addon.manifest.id.to_owned(),
        },
    )
}

fn addon_install_error_effects(addon: &Descriptor, error: OtherError) -> Effects {
    addon_action_error_effects(
        error,
        Event::AddonInstalled {
            transport_url: addon.transport_url.to_owned(),
            id: addon.manifest.id.to_owned(),
        },
    )
}

fn addon_action_error_effects(error: OtherError, source: Event) -> Effects {
    Effects::msg(Msg::Event(Event::Error {
        error: CtxError::from(error),
        source: Box::new(source),
    }))
    .unchanged()
}
