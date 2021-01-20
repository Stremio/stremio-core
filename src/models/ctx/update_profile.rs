use crate::constants::{OFFICIAL_ADDONS, PROFILE_STORAGE_KEY};
use crate::models::ctx::{CtxError, CtxStatus, OtherError};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, Effects, Env};
use crate::types::addon::Descriptor;
use crate::types::api::{fetch_api, APIRequest, APIResult, CollectionResponse, SuccessResponse};
use crate::types::profile::{AuthKey, Profile, Settings};
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};

pub fn update_profile<E: Env + 'static>(
    profile: &mut Profile,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
            let next_profile = Profile::default();
            if *profile != next_profile {
                *profile = next_profile;
                Effects::msg(Msg::Internal(Internal::ProfileChanged))
            } else {
                Effects::none().unchanged()
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::PushUserToAPI)) => {
            // TODO implement
            Effects::msg(Msg::Event(Event::UserPushedToAPI { uid: profile.uid() })).unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::PullUserFromAPI)) => {
            // TODO implement
            Effects::msg(Msg::Event(Event::UserPulledFromAPI { uid: profile.uid() })).unchanged()
        }
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
        Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI)) => {
            match profile.auth_key() {
                Some(auth_key) => Effects::one(pull_addons_from_api::<E>(auth_key)).unchanged(),
                _ => {
                    // TODO Does this code needs to be moved ?
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
                    let transport_urls = next_addons
                        .iter()
                        .map(|addon| &addon.transport_url)
                        .cloned()
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
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon))) => {
            if !profile.addons.contains(addon) {
                if !addon.manifest.behavior_hints.configuration_required {
                    let addon_position = profile
                        .addons
                        .iter()
                        .map(|addon| &addon.transport_url)
                        .position(|transport_url| *transport_url == addon.transport_url);
                    if let Some(addon_position) = addon_position {
                        profile.addons[addon_position] = addon.to_owned();
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
                    }))
                    .join(push_to_api_effects)
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::msg(Msg::Event(Event::Error {
                        error: CtxError::from(OtherError::AddonConfigurationRequired),
                        source: Box::new(Event::AddonInstalled {
                            transport_url: addon.transport_url.to_owned(),
                        }),
                    }))
                    .unchanged()
                }
            } else {
                Effects::msg(Msg::Event(Event::Error {
                    error: CtxError::from(OtherError::AddonAlreadyInstalled),
                    source: Box::new(Event::AddonInstalled {
                        transport_url: addon.transport_url.to_owned(),
                    }),
                }))
                .unchanged()
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(addon))) => {
            let addon_position = profile
                .addons
                .iter()
                .map(|addon| &addon.transport_url)
                .position(|transport_url| *transport_url == addon.transport_url);
            if let Some(addon_position) = addon_position {
                if !profile.addons[addon_position].flags.protected {
                    profile.addons.remove(addon_position);
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
                    }))
                    .join(push_to_api_effects)
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::msg(Msg::Event(Event::Error {
                        error: CtxError::from(OtherError::AddonIsProtected),
                        source: Box::new(Event::AddonUninstalled {
                            transport_url: addon.transport_url.to_owned(),
                        }),
                    }))
                    .unchanged()
                }
            } else {
                Effects::msg(Msg::Event(Event::Error {
                    error: CtxError::from(OtherError::AddonNotInstalled),
                    source: Box::new(Event::AddonUninstalled {
                        transport_url: addon.transport_url.to_owned(),
                    }),
                }))
                .unchanged()
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::UpdateSettings(settings))) => {
            if profile.settings != *settings {
                profile.settings = settings.to_owned();
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
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok((auth, addons, _)))
                if loading_auth_request == auth_request =>
            {
                let next_proifle = Profile {
                    auth: Some(auth.to_owned()),
                    addons: addons.to_owned(),
                    settings: Settings::default(),
                };
                if *profile != next_proifle {
                    *profile = next_proifle;
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
        )) if profile.auth_key() == Some(auth_key) => match result {
            Ok(addons) => {
                let transport_urls = addons
                    .iter()
                    .map(|addon| &addon.transport_url)
                    .cloned()
                    .collect();
                if profile.addons != *addons {
                    profile.addons = addons.to_owned();
                    Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { transport_urls }))
                        .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { transport_urls }))
                        .unchanged()
                }
            }
            Err(error) => Effects::msg(Msg::Event(Event::Error {
                error: error.to_owned(),
                source: Box::new(Event::AddonsPulledFromAPI {
                    transport_urls: Default::default(),
                }),
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
    fetch_api::<E, _, SuccessResponse>(&request)
        .map_err(CtxError::from)
        .and_then(|result| match result {
            APIResult::Ok { result } => future::ok(result),
            APIResult::Err { error } => future::err(CtxError::from(error)),
        })
        .map(move |result| match result {
            Ok(_) => Msg::Event(Event::AddonsPushedToAPI { transport_urls }),
            Err(error) => Msg::Event(Event::Error {
                error,
                source: Box::new(Event::AddonsPushedToAPI { transport_urls }),
            }),
        })
        .boxed_local()
        .into()
}

fn pull_addons_from_api<E: Env + 'static>(auth_key: &AuthKey) -> Effect {
    let request = APIRequest::AddonCollectionGet {
        auth_key: auth_key.to_owned(),
        update: true,
    };
    fetch_api::<E, _, _>(&request)
        .map_err(CtxError::from)
        .and_then(|result| match result {
            APIResult::Ok { result } => future::ok(result),
            APIResult::Err { error } => future::err(CtxError::from(error)),
        })
        .map_ok(|CollectionResponse { addons, .. }| addons)
        .map(move |result| Msg::Internal(Internal::AddonsAPIResult(request, result)))
        .boxed_local()
        .into()
}

fn push_profile_to_storage<E: Env + 'static>(profile: &Profile) -> Effect {
    E::set_storage(PROFILE_STORAGE_KEY, Some(profile))
        .map(enclose!((profile.uid() => uid) move |result| match result {
            Ok(_) => Msg::Event(Event::ProfilePushedToStorage { uid }),
            Err(error) => Msg::Event(Event::Error {
                error: CtxError::from(error),
                source: Box::new(Event::ProfilePushedToStorage { uid }),
            })
        }))
        .boxed_local()
        .into()
}
