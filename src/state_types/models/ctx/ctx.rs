use super::library::{LibraryLoadable, LibraryRequest};
use super::profile::{Profile, ProfileLoadable, ProfileRequest};
use crate::constants::OFFICIAL_ADDONS;
use crate::state_types::msg::{Action, ActionCtx, ActionLoad, Event, Internal, Msg};
use crate::state_types::{Effect, Effects, Environment, Update};
use crate::types::addons::Descriptor;
use crate::types::{LibBucket, LibItem, LibItemState};
use chrono::Datelike;
use derivative::Derivative;
use enclose::enclose;
use futures::future::Future;
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Derivative, Clone, Serialize)]
#[derivative(Default, Debug)]
pub struct Ctx<Env: Environment> {
    pub profile: ProfileLoadable,
    #[serde(skip)]
    pub library: LibraryLoadable,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    env: PhantomData<Env>,
}

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Ctx)) => {
                self.profile = ProfileLoadable::Loading {
                    request: ProfileRequest::Storage,
                    content: self.profile.content().to_owned(),
                };
                Effects::one(Box::new(ProfileLoadable::pull_from_storage::<Env>().then(
                    |result| Ok(Msg::Internal(Internal::ProfileStorageResult(result))),
                )))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Authenticate(auth_request))) => {
                let auth_request = auth_request.to_owned();
                self.profile = ProfileLoadable::Loading {
                    request: ProfileRequest::API(auth_request.to_owned()),
                    content: self.profile.content().to_owned(),
                };
                Effects::one(Box::new(
                    ProfileLoadable::authenticate::<Env>(&auth_request).then(move |result| {
                        Ok(Msg::Internal(Internal::AuthResult(auth_request, result)))
                    }),
                ))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
                let uid = self.profile.uid();
                let session_effects = match &self.profile.content().auth {
                    Some(auth) => Effects::one(Box::new(
                        ProfileLoadable::delete_session::<Env>(&auth.key).then(
                            enclose!((uid) move |result| {
                                match result {
                                    Ok(_) => Ok(Msg::Event(Event::SessionDeleted { uid })),
                                    Err(error) => Err(Msg::Event(Event::Error {
                                        error,
                                        event: Box::new(Event::SessionDeleted { uid }),
                                    })),
                                }
                            }),
                        ),
                    ))
                    .unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let next_proifle = ProfileLoadable::default();
                let profile_effects = if next_proifle.ne(&self.profile) {
                    self.profile = next_proifle;
                    Effects::msg(Msg::Internal(Internal::ProfileChanged))
                } else {
                    Effects::none().unchanged()
                };
                let next_library = LibraryLoadable::default();
                let library_effects = if next_library.ne(&self.library) {
                    self.library = next_library;
                    Effects::msg(Msg::Internal(Internal::LibraryChanged))
                } else {
                    Effects::none().unchanged()
                };
                Effects::msg(Msg::Event(Event::UserLoggedOut { uid }))
                    .unchanged()
                    .join(session_effects)
                    .join(profile_effects)
                    .join(library_effects)
            }
            Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon))) => {
                let profile_content = self.profile.content_mut();
                let addon_position = profile_content
                    .addons
                    .iter()
                    .map(|addon| &addon.transport_url)
                    .position(|transport_url| transport_url.eq(&addon.transport_url));
                if let Some(addon_position) = addon_position {
                    profile_content.addons.remove(addon_position);
                };
                profile_content.addons.push(addon.to_owned());
                Effects::msg(Msg::Event(Event::AddonInstalled {
                    uid: self.profile.uid(),
                    addon: addon.to_owned(),
                }))
                .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
            }
            Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(transport_url))) => {
                let profile_content = self.profile.content_mut();
                let addon_position = profile_content.addons.iter().position(|addon| {
                    addon.transport_url.eq(transport_url) && !addon.flags.protected
                });
                if let Some(addon_position) = addon_position {
                    let addon = profile_content.addons[addon_position].to_owned();
                    profile_content.addons.remove(addon_position);
                    Effects::msg(Msg::Event(Event::AddonUninstalled {
                        uid: self.profile.uid(),
                        addon,
                    }))
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::UpdateSettings(settings))) => {
                let mut profile_content = self.profile.content_mut();
                if profile_content.settings.ne(settings) {
                    profile_content.settings = settings.to_owned();
                    Effects::msg(Msg::Event(Event::SettingsUpdated {
                        uid: self.profile.uid(),
                    }))
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(meta_item))) => {
                let mut lib_item = LibItem {
                    id: meta_item.id.to_owned(),
                    type_name: meta_item.type_name.to_owned(),
                    name: meta_item.name.to_owned(),
                    poster: meta_item.poster.to_owned(),
                    background: None,
                    logo: meta_item.logo.to_owned(),
                    poster_shape: meta_item.poster_shape.to_owned(),
                    year: if let Some(released) = &meta_item.released {
                        Some(released.year().to_string())
                    } else if let Some(release_info) = &meta_item.release_info {
                        Some(release_info.to_owned())
                    } else {
                        None
                    },
                    removed: false,
                    temp: false,
                    ctime: Some(Env::now()),
                    mtime: Env::now(),
                    state: LibItemState::default(),
                };
                if let Some(LibItem { ctime, state, .. }) = self.library.get_item(&meta_item.id) {
                    lib_item.state = state.to_owned();
                    if let Some(ctime) = ctime {
                        lib_item.ctime = Some(ctime.to_owned());
                    };
                };
                Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item))).unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::RemoveFromLibrary(id))) => {
                if let Some(lib_item) = self.library.get_item(id) {
                    let mut lib_item = lib_item.to_owned();
                    lib_item.removed = true;
                    Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item))).unchanged()
                } else {
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::PushUserToAPI)) => {
                // TODO implement
                Effects::msg(Msg::Event(Event::UserPushedToAPI {
                    uid: self.profile.uid(),
                }))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::PullUserFromAPI)) => {
                // TODO implement
                Effects::msg(Msg::Event(Event::UserPulledFromAPI {
                    uid: self.profile.uid(),
                }))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::PushAddonsToAPI)) => {
                match &self.profile.content().auth {
                    Some(auth) => {
                        let uid = self.profile.uid();
                        Effects::one(Box::new(
                            ProfileLoadable::push_addons_to_api::<Env>(
                                &auth.key,
                                &self.profile.content().addons,
                            )
                            .then(move |result| match result {
                                Ok(_) => Ok(Msg::Event(Event::AddonsPushedToAPI { uid })),
                                Err(error) => Ok(Msg::Event(Event::Error {
                                    error,
                                    event: Box::new(Event::AddonsPushedToAPI { uid }),
                                })),
                            }),
                        ))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI)) => {
                match &self.profile.content().auth {
                    Some(auth) => {
                        let auth_key = auth.key.to_owned();
                        Effects::one(Box::new(
                            ProfileLoadable::pull_addons_from_api::<Env>(&auth_key, true).then(
                                move |result| {
                                    Ok(Msg::Internal(Internal::AddonsAPIResult(auth_key, result)))
                                },
                            ),
                        ))
                        .unchanged()
                    }
                    _ => {
                        let next_addons = self
                            .profile
                            .content()
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
                        let uid = self.profile.uid();
                        let mut profile_content = self.profile.content_mut();
                        if profile_content.addons.ne(&next_addons) {
                            profile_content.addons = next_addons;
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { uid }))
                                .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                        } else {
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI { uid })).unchanged()
                        }
                    }
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI)) => {
                match (&self.profile.content().auth, &self.library) {
                    (Some(auth), LibraryLoadable::Ready(bucket)) => {
                        let uid = bucket.uid.to_owned();
                        Effects::one(Box::new(
                            LibraryLoadable::sync_with_api::<Env>(&auth.key, bucket.to_owned())
                                .then(move |result| {
                                    Ok(Msg::Internal(Internal::LibrarySyncResult(uid, result)))
                                }),
                        ))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::UpdateLibraryItem(lib_item)) => match &mut self.library {
                LibraryLoadable::Ready(ref mut bucket) => {
                    let mut lib_item = lib_item.to_owned();
                    lib_item.mtime = Env::now();
                    let uid = bucket.uid.to_owned();
                    let persist_effect: Effect = Box::new(
                        LibraryLoadable::update_and_persist::<Env>(
                            bucket,
                            LibBucket::new(uid.to_owned(), vec![lib_item.to_owned()]),
                        )
                        .then(enclose!((uid) move |result| match result {
                            Ok(_) => Ok(Msg::Event(Event::LibraryPushedToStorage { uid })),
                            Err(error) => Err(Msg::Event(Event::Error {
                                error,
                                event: Box::new(Event::LibraryPushedToStorage { uid }),
                            })),
                        })),
                    );
                    let push_effect =
                        self.profile
                            .content()
                            .auth
                            .as_ref()
                            .map(move |auth| -> Effect {
                                Box::new(
                                    LibraryLoadable::push_to_api::<Env>(&auth.key, vec![lib_item])
                                        .then(move |result| match result {
                                            Ok(_) => {
                                                Ok(Msg::Event(Event::LibraryPushedToAPI { uid }))
                                            }
                                            Err(error) => Err(Msg::Event(Event::Error {
                                                error,
                                                event: Box::new(Event::LibraryPushedToAPI { uid }),
                                            })),
                                        }),
                                )
                            });
                    let library_effects = if let Some(push_effect) = push_effect {
                        Effects::many(vec![persist_effect, push_effect])
                    } else {
                        Effects::one(persist_effect)
                    };
                    library_effects.join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::ProfileStorageResult(result)) => match &self.profile {
                ProfileLoadable::Loading {
                    request: ProfileRequest::Storage,
                    ..
                } => match result {
                    Ok(profile) => {
                        let next_proifle = ProfileLoadable::Ready {
                            content: profile.to_owned().unwrap_or_default(),
                        };
                        let profile_effects = if next_proifle.ne(&self.profile) {
                            self.profile = next_proifle;
                            Effects::msg(Msg::Internal(Internal::ProfileChanged))
                        } else {
                            Effects::none().unchanged()
                        };
                        let uid = self.profile.uid();
                        let next_library =
                            LibraryLoadable::Loading(uid.to_owned(), LibraryRequest::Storage);
                        let library_effects = if next_library.ne(&self.library) {
                            self.library = next_library;
                            Effects::one(Box::new(
                                LibraryLoadable::pull_from_storage::<Env>().then(|result| {
                                    Ok(Msg::Internal(Internal::LibraryStorageResult(result)))
                                }),
                            ))
                            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                        } else {
                            Effects::none().unchanged()
                        };
                        Effects::msg(Msg::Event(Event::ProfilePulledFromStorage { uid }))
                            .unchanged()
                            .join(profile_effects)
                            .join(library_effects)
                    }
                    Err(error) => {
                        self.profile = ProfileLoadable::Ready {
                            content: self.profile.content().to_owned(),
                        };
                        Effects::msg(Msg::Event(Event::Error {
                            error: error.to_owned(),
                            event: Box::new(Event::ProfilePulledFromStorage {
                                uid: self.profile.uid(),
                            }),
                        }))
                        .unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::AuthResult(auth_request, result)) => match &self.profile {
                ProfileLoadable::Loading {
                    request: ProfileRequest::API(loading_auth_request),
                    ..
                } if loading_auth_request.eq(auth_request) => match result {
                    Ok((auth, addons)) => {
                        self.profile = ProfileLoadable::Ready {
                            content: Profile {
                                auth: Some(auth.to_owned()),
                                addons: addons.to_owned(),
                                ..Profile::default()
                            },
                        };
                        let uid = self.profile.uid();
                        let next_library =
                            LibraryLoadable::Loading(uid.to_owned(), LibraryRequest::API);
                        let library_effects = if next_library.ne(&self.library) {
                            Effects::one(Box::new(
                                LibraryLoadable::pull_from_api::<Env>(&auth.key, vec![], true)
                                    .then(enclose!((uid) move |result| {
                                        Ok(Msg::Internal(Internal::LibraryAPIResult(uid, result)))
                                    })),
                            ))
                            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                        } else {
                            Effects::none().unchanged()
                        };
                        Effects::msg(Msg::Event(Event::UserAuthenticated {
                            uid,
                            auth_request: auth_request.to_owned(),
                        }))
                        .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                        .join(library_effects)
                    }
                    Err(error) => {
                        self.profile = ProfileLoadable::Ready {
                            content: self.profile.content().to_owned(),
                        };
                        Effects::msg(Msg::Event(Event::Error {
                            error: error.to_owned(),
                            event: Box::new(Event::UserAuthenticated {
                                uid: self.profile.uid(),
                                auth_request: auth_request.to_owned(),
                            }),
                        }))
                        .unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::AddonsAPIResult(auth_key, result))
                if self
                    .profile
                    .content()
                    .auth
                    .as_ref()
                    .map(|auth| &auth.key)
                    .eq(&Some(auth_key)) =>
            {
                match result {
                    Ok(addons) => {
                        let mut profile_content = self.profile.content_mut();
                        if profile_content.addons.ne(addons) {
                            profile_content.addons = addons.to_owned();
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                                uid: self.profile.uid(),
                            }))
                        } else {
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                                uid: self.profile.uid(),
                            }))
                            .unchanged()
                        }
                    }
                    Err(error) => Effects::msg(Msg::Event(Event::Error {
                        error: error.to_owned(),
                        event: Box::new(Event::AddonsPulledFromAPI {
                            uid: self.profile.uid(),
                        }),
                    }))
                    .unchanged(),
                }
            }
            Msg::Internal(Internal::LibraryStorageResult(result)) => match &self.library {
                LibraryLoadable::Loading(uid, LibraryRequest::Storage) => {
                    let uid = uid.to_owned();
                    match result {
                        Ok((recent_bucket, other_bucket)) => {
                            let mut bucket = LibBucket::new(uid.to_owned(), vec![]);
                            if let Some(recent_bucket) = recent_bucket {
                                bucket.merge(recent_bucket.to_owned())
                            };
                            if let Some(other_bucket) = other_bucket {
                                bucket.merge(other_bucket.to_owned())
                            };
                            self.library = LibraryLoadable::Ready(bucket);
                            Effects::msg(Msg::Event(Event::LibraryPulledFromStorage {
                                uid: uid.to_owned(),
                            }))
                            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                        }
                        Err(error) => {
                            self.library =
                                LibraryLoadable::Ready(LibBucket::new(uid.to_owned(), vec![]));
                            Effects::msg(Msg::Event(Event::Error {
                                error: error.to_owned(),
                                event: Box::new(Event::LibraryPulledFromStorage {
                                    uid: uid.to_owned(),
                                }),
                            }))
                            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                        }
                    }
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::LibraryAPIResult(uid, result)) => match &self.library {
                LibraryLoadable::Loading(loading_uid, LibraryRequest::API)
                    if loading_uid.eq(&uid) =>
                {
                    match result {
                        Ok(items) => {
                            self.library = LibraryLoadable::Ready(LibBucket::new(
                                uid.to_owned(),
                                items.to_owned(),
                            ));
                            Effects::msg(Msg::Event(Event::LibrarySyncedWithAPI {
                                uid: uid.to_owned(),
                            }))
                            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                        }
                        Err(error) => {
                            self.library =
                                LibraryLoadable::Ready(LibBucket::new(uid.to_owned(), vec![]));
                            Effects::msg(Msg::Event(Event::Error {
                                error: error.to_owned(),
                                event: Box::new(Event::LibrarySyncedWithAPI {
                                    uid: uid.to_owned(),
                                }),
                            }))
                            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                        }
                    }
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::LibrarySyncResult(uid, result)) => match &mut self.library {
                LibraryLoadable::Ready(ref mut bucket) if bucket.uid.eq(uid) => {
                    let uid = uid.to_owned();
                    match result {
                        Ok(items) => Effects::msg(Msg::Event(Event::LibrarySyncedWithAPI {
                            uid: uid.to_owned(),
                        }))
                        .join(Effects::one(Box::new(
                            LibraryLoadable::update_and_persist::<Env>(
                                bucket,
                                LibBucket::new(uid.to_owned(), items.to_owned()),
                            )
                            .then(move |result| match result {
                                Ok(_) => Ok(Msg::Event(Event::LibraryPushedToStorage { uid })),
                                Err(error) => Err(Msg::Event(Event::Error {
                                    error,
                                    event: Box::new(Event::LibraryPushedToStorage { uid }),
                                })),
                            }),
                        )))
                        .join(Effects::msg(Msg::Internal(Internal::LibraryChanged))),
                        Err(error) => Effects::msg(Msg::Event(Event::Error {
                            error: error.to_owned(),
                            event: Box::new(Event::LibrarySyncedWithAPI { uid }),
                        }))
                        .unchanged(),
                    }
                }
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        }
    }
}
