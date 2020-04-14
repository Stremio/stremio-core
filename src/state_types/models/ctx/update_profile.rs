use super::{fetch_api, CtxError, CtxRequest, CtxStatus};
use crate::constants::{OFFICIAL_ADDONS, PROFILE_STORAGE_KEY};
use crate::state_types::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::state_types::{Effects, Environment};
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, CollectionResponse, SuccessResponse};
use crate::types::profile::{Profile, Settings};
use enclose::enclose;
use futures::Future;
use std::ops::Deref;

pub fn update_profile<Env: Environment + 'static>(
    profile: &mut Profile,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
            let next_profile = Profile::default();
            if next_profile.ne(profile) {
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
        Msg::Action(Action::Ctx(ActionCtx::PushAddonsToAPI)) => {
            match &profile.auth {
                Some(auth) => Effects::one(Box::new(
                    fetch_api::<Env, _, SuccessResponse>(&APIRequest::AddonCollectionSet {
                        auth_key: auth.key.to_owned(),
                        addons: profile.addons.to_owned(),
                    })
                    .map(enclose!((profile.uid() => uid) move |_| {
                        Msg::Event(Event::AddonsPushedToAPI { uid })
                    }))
                    .map_err(enclose!((profile.uid() => uid) move |error| {
                        Msg::Event(Event::Error {
                            error,
                            source: Box::new(Event::AddonsPushedToAPI { uid }),
                        })
                    })),
                ))
                .unchanged(),
                _ => {
                    // TODO Consider return error event for user not logged in
                    Effects::none().unchanged()
                }
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI)) => {
            match &profile.auth {
                Some(auth) => Effects::one(Box::new(
                    fetch_api::<Env, _, _>(&APIRequest::AddonCollectionGet {
                        auth_key: auth.key.to_owned(),
                        update: true,
                    })
                    .map(|CollectionResponse { addons, .. }| addons)
                    .then(enclose!((auth.key.to_owned() => auth_key)
                        move |result| Ok(Msg::Internal(Internal::AddonsAPIResult(auth_key, result)))
                    )),
                ))
                .unchanged(),
                _ => {
                    // TODO Does this code need to be moved ?
                    let next_addons = profile
                        .addons
                        .iter()
                        .map(|profile_addon| {
                            OFFICIAL_ADDONS
                                .iter()
                                .find(|Descriptor { manifest, .. }| {
                                    manifest.id.eq(&profile_addon.manifest.id)
                                        && manifest.version.gt(&profile_addon.manifest.version)
                                })
                                .map(|official_addon| Descriptor {
                                    transport_url: official_addon.transport_url.to_owned(),
                                    manifest: official_addon.manifest.to_owned(),
                                    flags: profile_addon.flags.to_owned(),
                                })
                                .unwrap_or_else(|| profile_addon.to_owned())
                        })
                        .collect();
                    if profile.addons.ne(&next_addons) {
                        profile.addons = next_addons;
                        Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                            uid: profile.uid(),
                        }))
                        .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
                    } else {
                        Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                            uid: profile.uid(),
                        }))
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
                    .position(|transport_url| transport_url.eq(&addon.transport_url));
                if let Some(addon_position) = addon_position {
                    profile.addons[addon_position] = addon.to_owned();
                } else {
                    profile.addons.push(addon.to_owned());
                };
                Effects::msg(Msg::Event(Event::AddonInstalled {
                    uid: profile.uid(),
                    addon: addon.to_owned(),
                }))
                .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
            } else {
                Effects::msg(Msg::Event(Event::Error {
                    error: CtxError::from("Addon already installed"),
                    source: Box::new(Event::AddonInstalled {
                        uid: profile.uid(),
                        addon: addon.to_owned(),
                    }),
                }))
                .unchanged()
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(transport_url))) => {
            let addon_position = profile
                .addons
                .iter()
                .position(|addon| addon.transport_url.eq(transport_url) && !addon.flags.protected);
            if let Some(addon_position) = addon_position {
                let addon = profile.addons[addon_position].to_owned();
                profile.addons.remove(addon_position);
                Effects::msg(Msg::Event(Event::AddonUninstalled {
                    uid: profile.uid(),
                    addon,
                }))
                .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
            } else {
                // TODO Consider return error event if addon is not installed
                // TODO Consider return error event if addon is protected
                Effects::none().unchanged()
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::UpdateSettings(settings))) => {
            if profile.settings.ne(settings) {
                profile.settings = settings.to_owned();
                Effects::msg(Msg::Event(Event::SettingsUpdated { uid: profile.uid() }))
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
            } else {
                Effects::msg(Msg::Event(Event::SettingsUpdated { uid: profile.uid() })).unchanged()
            }
        }
        Msg::Internal(Internal::CtxStorageResult(result)) => match (status, result.deref()) {
            (CtxStatus::Loading(CtxRequest::Storage), Ok((result_profile, _, _))) => {
                let next_proifle = result_profile.to_owned().unwrap_or_default();
                if next_proifle.ne(profile) {
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
                if next_proifle.ne(profile) {
                    *profile = next_proifle;
                    Effects::msg(Msg::Internal(Internal::ProfileChanged(false)))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::AddonsAPIResult(auth_key, result))
            if profile
                .auth
                .as_ref()
                .map(|auth| &auth.key)
                .eq(&Some(auth_key)) =>
        {
            match result {
                Ok(addons) => {
                    if profile.addons.ne(addons) {
                        profile.addons = addons.to_owned();
                        Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                            uid: profile.uid(),
                        }))
                        .join(Effects::msg(Msg::Internal(Internal::ProfileChanged(false))))
                    } else {
                        Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                            uid: profile.uid(),
                        }))
                        .unchanged()
                    }
                }
                Err(error) => Effects::msg(Msg::Event(Event::Error {
                    error: error.to_owned(),
                    source: Box::new(Event::AddonsPulledFromAPI { uid: profile.uid() }),
                }))
                .unchanged(),
            }
        }
        Msg::Internal(Internal::ProfileChanged(persisted)) if !persisted => Effects::one(Box::new(
            Env::set_storage(PROFILE_STORAGE_KEY, Some(profile))
                .map_err(CtxError::from)
                .map(enclose!((profile.uid() => uid) move |_| {
                    Msg::Event(Event::ProfilePushedToStorage { uid })
                }))
                .map_err(enclose!((profile.uid() => uid) move |error| {
                    Msg::Event(Event::Error {
                        error,
                        source: Box::new(Event::ProfilePushedToStorage { uid }),
                    })
                })),
        ))
        .unchanged(),
        _ => Effects::none().unchanged(),
    }
}
