use super::{fetch_api, CtxError, CtxRequest, CtxStatus, OtherError};
use crate::constants::{OFFICIAL_ADDONS, PROFILE_STORAGE_KEY};
use crate::state_types::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::state_types::{Effect, Effects, Environment};
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, CollectionResponse, SuccessResponse};
use crate::types::profile::{Profile, Settings};
use enclose::enclose;
use futures::Future;

pub fn update_profile<Env: Environment + 'static>(
    profile: &mut Profile,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
            let next_profile = Profile::default();
            if *profile != next_profile {
                *profile = next_profile;
                Effects::msg(Msg::Internal(Internal::ProfileChanged(false)))
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
        Msg::Action(Action::Ctx(ActionCtx::PushAddonsToAPI)) => match &profile.auth {
            Some(auth) => Effects::one(push_addons_to_api::<Env>(
                profile.addons.to_owned(),
                &auth.key,
            ))
            .unchanged(),
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
            match &profile.auth {
                Some(auth) => Effects::one(pull_addons_from_api::<Env>(&auth.key)).unchanged(),
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
                            .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
                    } else {
                        Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { transport_urls }))
                            .unchanged()
                    }
                }
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon))) => {
            if !profile.addons.contains(addon) {
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
                let push_to_api_effects = match &profile.auth {
                    Some(auth) => Effects::one(push_addons_to_api::<Env>(
                        profile.addons.to_owned(),
                        &auth.key,
                    ))
                    .unchanged(),
                    _ => Effects::none().unchanged(),
                };
                Effects::msg(Msg::Event(Event::AddonInstalled {
                    transport_url: addon.transport_url.to_owned(),
                }))
                .join(push_to_api_effects)
                .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
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
        Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(transport_url))) => {
            let addon_position = profile
                .addons
                .iter()
                .position(|addon| addon.transport_url == *transport_url);
            if let Some(addon_position) = addon_position {
                if !profile.addons[addon_position].flags.protected {
                    profile.addons.remove(addon_position);
                    let push_to_api_effects = match &profile.auth {
                        Some(auth) => Effects::one(push_addons_to_api::<Env>(
                            profile.addons.to_owned(),
                            &auth.key,
                        ))
                        .unchanged(),
                        _ => Effects::none().unchanged(),
                    };
                    Effects::msg(Msg::Event(Event::AddonUninstalled {
                        transport_url: transport_url.to_owned(),
                    }))
                    .join(push_to_api_effects)
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
                } else {
                    Effects::msg(Msg::Event(Event::Error {
                        error: CtxError::from(OtherError::AddonIsProtected),
                        source: Box::new(Event::AddonUninstalled {
                            transport_url: transport_url.to_owned(),
                        }),
                    }))
                    .unchanged()
                }
            } else {
                Effects::msg(Msg::Event(Event::Error {
                    error: CtxError::from(OtherError::AddonNotInstalled),
                    source: Box::new(Event::AddonUninstalled {
                        transport_url: transport_url.to_owned(),
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
                .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
            } else {
                Effects::msg(Msg::Event(Event::SettingsUpdated {
                    settings: settings.to_owned(),
                }))
                .unchanged()
            }
        }
        Msg::Internal(Internal::ProfileChanged(persisted)) if !persisted => {
            Effects::one(push_profile_to_storage::<Env>(profile)).unchanged()
        }
        Msg::Internal(Internal::CtxStorageResult(result)) => match (status, result) {
            (CtxStatus::Loading(CtxRequest::Storage), Ok((result_profile, _, _))) => {
                let next_proifle = result_profile.to_owned().unwrap_or_default();
                if *profile != next_proifle {
                    *profile = next_proifle;
                    Effects::msg(Msg::Internal(Internal::ProfileChanged(true)))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(CtxRequest::API(loading_auth_request)), Ok((auth, addons, _)))
                if loading_auth_request == auth_request =>
            {
                let next_proifle = Profile {
                    auth: Some(auth.to_owned()),
                    addons: addons.to_owned(),
                    settings: Settings::default(),
                };
                if *profile != next_proifle {
                    *profile = next_proifle;
                    Effects::msg(Msg::Internal(Internal::ProfileChanged(false)))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::AddonsAPIResult(
            APIRequest::AddonCollectionGet { auth_key, .. },
            result,
        )) if profile.auth.as_ref().map(|auth| &auth.key) == Some(auth_key) => match result {
            Ok(addons) => {
                let transport_urls = addons
                    .iter()
                    .map(|addon| &addon.transport_url)
                    .cloned()
                    .collect();
                if profile.addons != *addons {
                    profile.addons = addons.to_owned();
                    Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { transport_urls }))
                        .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
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

fn push_addons_to_api<Env: Environment + 'static>(
    addons: Vec<Descriptor>,
    auth_key: &str,
) -> Effect {
    let transport_urls = addons
        .iter()
        .map(|addon| &addon.transport_url)
        .cloned()
        .collect();
    let request = APIRequest::AddonCollectionSet {
        auth_key: auth_key.to_owned(),
        addons,
    };
    Box::new(
        fetch_api::<Env, _, SuccessResponse>(&request).then(move |result| match result {
            Ok(_) => Ok(Msg::Event(Event::AddonsPushedToAPI { transport_urls })),
            Err(error) => Err(Msg::Event(Event::Error {
                error,
                source: Box::new(Event::AddonsPushedToAPI { transport_urls }),
            })),
        }),
    )
}

fn pull_addons_from_api<Env: Environment + 'static>(auth_key: &str) -> Effect {
    let request = APIRequest::AddonCollectionGet {
        auth_key: auth_key.to_owned(),
        update: true,
    };
    Box::new(
        fetch_api::<Env, _, _>(&request)
            .map(|CollectionResponse { addons, .. }| addons)
            .then(move |result| Ok(Msg::Internal(Internal::AddonsAPIResult(request, result)))),
    )
}

fn push_profile_to_storage<Env: Environment + 'static>(profile: &Profile) -> Effect {
    Box::new(Env::set_storage(PROFILE_STORAGE_KEY, Some(profile)).then(
        enclose!((profile.uid() => uid) move |result| match result {
            Ok(_) => Ok(Msg::Event(Event::ProfilePushedToStorage { uid })),
            Err(error) => Err(Msg::Event(Event::Error {
                error: CtxError::from(error),
                source: Box::new(Event::ProfilePushedToStorage { uid }),
            }))
        }),
    ))
}
