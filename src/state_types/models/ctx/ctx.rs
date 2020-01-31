use super::library::{LibraryLoadable, LibraryRequest};
use super::user::{User, UserLoadable, UserRequest};
use crate::constants::OFFICIAL_ADDONS;
use crate::state_types::msg::{Action, ActionCtx, ActionLoad, Event, Internal, Msg};
use crate::state_types::{Effect, Effects, Environment, Update};
use crate::types::addons::Descriptor;
use crate::types::api::APIRequest;
use crate::types::{LibBucket, LibItem, LibItemState, UID};
use chrono::Datelike;
use derivative::Derivative;
use futures::future::Future;
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Derivative, Clone, Serialize)]
#[derivative(Default, Debug)]
pub struct Ctx<Env: Environment> {
    #[serde(flatten)]
    pub user: UserLoadable,
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
                self.user = UserLoadable::Loading {
                    request: UserRequest::Storage,
                    content: self.user.content().to_owned(),
                };
                Effects::one(Box::new(UserLoadable::pull_from_storage::<Env>().then(
                    |result| Ok(Msg::Internal(Internal::UserStorageResult(result))),
                )))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Login { email, password })) => {
                let request = APIRequest::Login {
                    email: email.to_owned(),
                    password: password.to_owned(),
                };
                self.user = UserLoadable::Loading {
                    request: UserRequest::API(request.to_owned()),
                    content: self.user.content().to_owned(),
                };
                Effects::one(Box::new(UserLoadable::authenticate::<Env>(&request).then(
                    move |result| Ok(Msg::Internal(Internal::UserAuthResult(request, result))),
                )))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Register {
                email,
                password,
                gdpr_consent,
            })) => {
                let request = APIRequest::Register {
                    email: email.to_owned(),
                    password: password.to_owned(),
                    gdpr_consent: gdpr_consent.to_owned(),
                };
                self.user = UserLoadable::Loading {
                    request: UserRequest::API(request.to_owned()),
                    content: self.user.content().to_owned(),
                };
                Effects::one(Box::new(UserLoadable::authenticate::<Env>(&request).then(
                    move |result| Ok(Msg::Internal(Internal::UserAuthResult(request, result))),
                )))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
                let session_effects = match &self.user.content().auth {
                    Some(auth) => Effects::one(Box::new(
                        UserLoadable::delete_session::<Env>(&auth.key)
                            .map(|_| Msg::Event(Event::UserSessionDeleted))
                            .map_err(|error| Msg::Event(Event::Error(error))),
                    ))
                    .unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let next_user = UserLoadable::default();
                let user_effects = if next_user.ne(&self.user) {
                    self.user = next_user;
                    Effects::msg(Msg::Internal(Internal::UserChanged))
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
                Effects::msg(Msg::Event(Event::UserLoggedOut))
                    .unchanged()
                    .join(session_effects)
                    .join(user_effects)
                    .join(library_effects)
            }
            Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon))) => {
                let user_content = self.user.content_mut();
                let addon_position = user_content
                    .addons
                    .iter()
                    .map(|addon| &addon.transport_url)
                    .position(|transport_url| transport_url.eq(&addon.transport_url));
                if let Some(addon_position) = addon_position {
                    user_content.addons.remove(addon_position);
                };
                user_content.addons.push(addon.to_owned());
                Effects::msg(Msg::Internal(Internal::UserChanged))
                    .join(Effects::msg(Msg::Event(Event::AddonInstalled)))
            }
            Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(transport_url))) => {
                let user_content = self.user.content_mut();
                let addon_position = user_content.addons.iter().position(|addon| {
                    addon.transport_url.eq(transport_url) && !addon.flags.protected
                });
                if let Some(addon_position) = addon_position {
                    user_content.addons.remove(addon_position);
                    Effects::msg(Msg::Internal(Internal::UserChanged))
                        .join(Effects::msg(Msg::Event(Event::AddonUninstalled)))
                } else {
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::UpdateSettings(settings))) => {
                let mut user_content = self.user.content_mut();
                if user_content.settings.ne(settings) {
                    user_content.settings = settings.to_owned();
                    Effects::msg(Msg::Internal(Internal::UserChanged))
                        .join(Effects::msg(Msg::Event(Event::SettingsUpdated)))
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
            Msg::Action(Action::Ctx(ActionCtx::PushUserToAPI)) => Effects::none().unchanged(),
            Msg::Action(Action::Ctx(ActionCtx::PullUserFromAPI)) => Effects::none().unchanged(),
            Msg::Action(Action::Ctx(ActionCtx::PushAddonsToAPI)) => match &self.user.content().auth
            {
                Some(auth) => Effects::one(Box::new(
                    UserLoadable::push_addons_to_api::<Env>(&auth.key, &self.user.content().addons)
                        .map(|_| Msg::Event(Event::AddonsPushedToAPI))
                        .map_err(|error| Msg::Event(Event::Error(error))),
                ))
                .unchanged(),
                _ => Effects::none().unchanged(),
            },
            Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI)) => {
                match &self.user.content().auth {
                    Some(auth) => {
                        let auth_key = auth.key.to_owned();
                        Effects::one(Box::new(
                            UserLoadable::pull_addons_from_api::<Env>(&auth_key).then(
                                move |result| {
                                    Ok(Msg::Internal(Internal::UserAddonsAPIResult(
                                        auth_key, result,
                                    )))
                                },
                            ),
                        ))
                        .unchanged()
                    }
                    _ => {
                        let next_addons = self
                            .user
                            .content()
                            .addons
                            .iter()
                            .map(|user_addon| {
                                OFFICIAL_ADDONS
                                    .iter()
                                    .find(|Descriptor { manifest, .. }| {
                                        manifest.id.eq(&user_addon.manifest.id)
                                            && manifest.version.gt(&user_addon.manifest.version)
                                    })
                                    .map(|official_addon| Descriptor {
                                        transport_url: official_addon.transport_url.to_owned(),
                                        manifest: official_addon.manifest.to_owned(),
                                        flags: user_addon.flags.to_owned(),
                                    })
                                    .unwrap_or_else(|| user_addon.to_owned())
                            })
                            .collect();
                        let mut user_content = self.user.content_mut();
                        if user_content.addons.ne(&next_addons) {
                            user_content.addons = next_addons;
                            Effects::none()
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI)) => {
                match (&self.user.content().auth, &self.library) {
                    (Some(auth), LibraryLoadable::Ready(bucket)) => {
                        let uid = bucket.uid.to_owned();
                        Effects::one(Box::new(
                            LibraryLoadable::sync_with_api::<Env>(&auth, bucket.to_owned()).then(
                                move |result| {
                                    Ok(Msg::Internal(Internal::LibrarySyncResult(uid, result)))
                                },
                            ),
                        ))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::UserStorageResult(result)) => match &self.user {
                UserLoadable::Loading {
                    request: UserRequest::Storage,
                    ..
                } => match result {
                    Ok(user) => {
                        let next_user = UserLoadable::Ready {
                            content: user.to_owned().unwrap_or_default(),
                        };
                        let user_effects = if next_user.ne(&self.user) {
                            self.user = next_user;
                            Effects::msg(Msg::Internal(Internal::UserChanged))
                        } else {
                            Effects::none().unchanged()
                        };
                        let next_library = LibraryLoadable::Loading(
                            UID(self
                                .user
                                .content()
                                .auth
                                .as_ref()
                                .map(|auth| auth.user.id.to_owned())),
                            LibraryRequest::Storage,
                        );
                        let library_effects = if next_library.ne(&self.library) {
                            self.library = next_library;
                            Effects::msg(Msg::Internal(Internal::LibraryChanged)).join(
                                Effects::one(Box::new(
                                    LibraryLoadable::pull_from_storage::<Env>().then(|result| {
                                        Ok(Msg::Internal(Internal::LibraryStorageResult(result)))
                                    }),
                                )),
                            )
                        } else {
                            Effects::none().unchanged()
                        };
                        Effects::msg(Msg::Event(Event::UserPulledFromStorage))
                            .unchanged()
                            .join(user_effects)
                            .join(library_effects)
                    }
                    Err(error) => {
                        self.user = UserLoadable::Ready {
                            content: self.user.content().to_owned(),
                        };
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserAuthResult(api_request, result)) => match &self.user {
                UserLoadable::Loading {
                    request: UserRequest::API(loading_api_request),
                    ..
                } if loading_api_request.eq(api_request) => match result {
                    Ok((auth, addons)) => {
                        self.user = UserLoadable::Ready {
                            content: User {
                                auth: Some(auth.to_owned()),
                                addons: addons.to_owned(),
                                ..User::default()
                            },
                        };
                        let uid = UID(Some(auth.user.id.to_owned()));
                        let next_library =
                            LibraryLoadable::Loading(uid.to_owned(), LibraryRequest::API);
                        let library_effects = if next_library.ne(&self.library) {
                            Effects::msg(Msg::Internal(Internal::LibraryChanged)).join(
                                Effects::one(Box::new(
                                    LibraryLoadable::pull_from_api::<Env>(&auth.key, vec![], true)
                                        .then(move |result| {
                                            Ok(Msg::Internal(Internal::LibraryAPIResult(
                                                uid, result,
                                            )))
                                        }),
                                )),
                            )
                        } else {
                            Effects::none().unchanged()
                        };
                        Effects::msg(Msg::Event(Event::UserAuthenticated))
                            .join(Effects::msg(Msg::Internal(Internal::UserChanged)))
                            .join(library_effects)
                    }
                    Err(error) => {
                        self.user = UserLoadable::Ready {
                            content: self.user.content().to_owned(),
                        };
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserAddonsAPIResult(auth_key, result))
                if self
                    .user
                    .content()
                    .auth
                    .as_ref()
                    .map(|auth| &auth.key)
                    .eq(&Some(auth_key)) =>
            {
                match result {
                    Ok(addons) => {
                        let mut user_content = self.user.content_mut();
                        if user_content.addons.ne(addons) {
                            user_content.addons = addons.to_owned();
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI))
                        } else {
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI)).unchanged()
                        }
                    }
                    Err(error) => {
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                }
            }
            Msg::Internal(Internal::UpdateLibraryItem(lib_item)) => match &mut self.library {
                LibraryLoadable::Ready(ref mut bucket) => {
                    let mut lib_item = lib_item.to_owned();
                    lib_item.mtime = Env::now();
                    let push_effect = self.user.content().auth.as_ref().map(|auth| -> Effect {
                        Box::new(
                            LibraryLoadable::push_to_api::<Env>(
                                &auth.key,
                                vec![lib_item.to_owned()],
                            )
                            .map(|_| Msg::Event(Event::LibraryPushedToAPI))
                            .map_err(|error| Msg::Event(Event::Error(error))),
                        )
                    });
                    let persist_effect: Effect = Box::new(
                        LibraryLoadable::update_and_persist::<Env>(
                            bucket,
                            LibBucket::new(bucket.uid.to_owned(), vec![lib_item]),
                        )
                        .map(|_| Msg::Event(Event::LibraryPushedToStorage))
                        .map_err(|error| Msg::Event(Event::Error(error))),
                    );
                    let library_effects = if let Some(push_effect) = push_effect {
                        Effects::many(vec![persist_effect, push_effect])
                    } else {
                        Effects::one(persist_effect)
                    };
                    Effects::msg(Msg::Internal(Internal::LibraryChanged)).join(library_effects)
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::LibraryStorageResult(result)) => match &self.library {
                LibraryLoadable::Loading(uid, LibraryRequest::Storage) => match result {
                    Ok((recent_bucket, other_bucket)) => {
                        let mut bucket = LibBucket::new(uid.to_owned(), vec![]);
                        if let Some(recent_bucket) = recent_bucket {
                            bucket.merge(recent_bucket.to_owned())
                        };
                        if let Some(other_bucket) = other_bucket {
                            bucket.merge(other_bucket.to_owned())
                        };
                        self.library = LibraryLoadable::Ready(bucket);
                        Effects::msg(Msg::Internal(Internal::LibraryChanged))
                            .join(Effects::msg(Msg::Event(Event::LibraryPulledFromStorage)))
                    }
                    Err(error) => {
                        self.library =
                            LibraryLoadable::Ready(LibBucket::new(uid.to_owned(), vec![]));
                        Effects::msg(Msg::Internal(Internal::LibraryChanged))
                            .join(Effects::msg(Msg::Event(Event::Error(error.to_owned()))))
                    }
                },
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
                            Effects::msg(Msg::Internal(Internal::LibraryChanged))
                                .join(Effects::msg(Msg::Event(Event::LibrarySyncedWithAPI)))
                        }
                        Err(error) => {
                            self.library =
                                LibraryLoadable::Ready(LibBucket::new(uid.to_owned(), vec![]));
                            Effects::msg(Msg::Internal(Internal::LibraryChanged))
                                .join(Effects::msg(Msg::Event(Event::Error(error.to_owned()))))
                        }
                    }
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::LibrarySyncResult(uid, result)) => match &mut self.library {
                LibraryLoadable::Ready(ref mut bucket) if bucket.uid.eq(uid) => match result {
                    Ok(items) => Effects::msg(Msg::Internal(Internal::LibraryChanged))
                        .join(Effects::msg(Msg::Event(Event::LibrarySyncedWithAPI)))
                        .join(Effects::one(Box::new(
                            LibraryLoadable::update_and_persist::<Env>(
                                bucket,
                                LibBucket::new(uid.to_owned(), items.to_owned()),
                            )
                            .map(|_| Msg::Event(Event::LibraryPushedToStorage))
                            .map_err(|error| Msg::Event(Event::Error(error))),
                        ))),
                    Err(error) => {
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        }
    }
}
