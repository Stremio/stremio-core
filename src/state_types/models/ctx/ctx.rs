use super::{
    authenticate, delete_session, pull_addons_from_api, pull_library_from_api,
    pull_library_from_storage, pull_profile_from_storage, push_addons_to_api,
    push_library_borrowed_to_storage, push_library_to_api, push_profile_to_storage,
    sync_library_with_api, update_and_persist_library, CtxError,
};
use crate::constants::OFFICIAL_ADDONS;
use crate::state_types::msg::{Action, ActionCtx, ActionLoad, Event, Internal, Msg};
use crate::state_types::{Effect, Effects, Environment, Update};
use crate::types::addons::Descriptor;
use crate::types::api::AuthRequest;
use crate::types::profile::{Profile, Settings};
use crate::types::{LibBucket, LibItem, LibItemState};
use derivative::Derivative;
use enclose::enclose;
use futures::Future;
use serde::Serialize;
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq)]
pub enum CtxRequest {
    Storage,
    API(AuthRequest),
}

#[derive(Derivative, Clone, PartialEq, Serialize)]
#[derivative(Debug)]
#[serde(untagged)]
pub enum Ctx<Env: Environment> {
    Loading {
        #[serde(skip)]
        request: CtxRequest,
        profile: Profile,
        #[serde(skip)]
        library: LibBucket,
        #[serde(skip)]
        #[derivative(Debug = "ignore")]
        env: PhantomData<Env>,
    },
    Ready {
        profile: Profile,
        #[serde(skip)]
        library: LibBucket,
        #[serde(skip)]
        #[derivative(Debug = "ignore")]
        env: PhantomData<Env>,
    },
}

impl<Env: Environment + 'static> Ctx<Env> {
    pub fn profile(&self) -> &Profile {
        match &self {
            Ctx::Loading { profile, .. } | Ctx::Ready { profile, .. } => &profile,
        }
    }
    pub fn library(&self) -> &LibBucket {
        match &self {
            Ctx::Loading { library, .. } | Ctx::Ready { library, .. } => &library,
        }
    }
    fn profile_mut(&mut self) -> &mut Profile {
        match self {
            Ctx::Loading { profile, .. } | Ctx::Ready { profile, .. } => profile,
        }
    }
    fn library_mut(&mut self) -> &mut LibBucket {
        match self {
            Ctx::Loading { library, .. } | Ctx::Ready { library, .. } => library,
        }
    }
    fn loading(&self, request: CtxRequest) -> Self {
        let (profile, library) = match &self {
            Ctx::Loading {
                profile, library, ..
            }
            | Ctx::Ready {
                profile, library, ..
            } => (profile, library),
        };
        Ctx::Loading {
            request,
            profile: profile.to_owned(),
            library: library.to_owned(),
            env: PhantomData,
        }
    }
    fn ready(&self) -> Self {
        let (profile, library) = match &self {
            Ctx::Loading {
                profile, library, ..
            }
            | Ctx::Ready {
                profile, library, ..
            } => (profile, library),
        };
        Ctx::Ready {
            profile: profile.to_owned(),
            library: library.to_owned(),
            env: PhantomData,
        }
    }
}

impl<Env: Environment + 'static> Default for Ctx<Env> {
    fn default() -> Self {
        Ctx::Ready {
            profile: Profile::default(),
            library: LibBucket::default(),
            env: PhantomData,
        }
    }
}

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Ctx)) => {
                *self = self.loading(CtxRequest::Storage);
                Effects::one(Box::new(
                    pull_profile_from_storage::<Env>()
                        .join(pull_library_from_storage::<Env>())
                        .map(|(profile, (recent_bucket, other_bucket))| {
                            (profile, recent_bucket, other_bucket)
                        })
                        .then(|result| {
                            Ok(Msg::Internal(Internal::CtxStorageResult(Box::new(result))))
                        }),
                ))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Authenticate(auth_request))) => {
                *self = self.loading(CtxRequest::API(auth_request.to_owned()));
                Effects::one(Box::new(
                    authenticate::<Env>(&auth_request)
                        .and_then(|auth| {
                            pull_addons_from_api::<Env>(&auth.key, true)
                                .join(pull_library_from_api::<Env>(&auth.key, vec![], true))
                                .map(move |(addons, lib_items)| (auth, addons, lib_items))
                        })
                        .then(enclose!((auth_request) move |result| {
                            Ok(Msg::Internal(Internal::CtxAuthResult(auth_request, result)))
                        })),
                ))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
                let uid = self.profile().uid();
                let session_effects = match &self.profile().auth {
                    Some(auth) => Effects::one(Box::new(delete_session::<Env>(&auth.key).then(
                        enclose!((uid) move |result| match result {
                            Ok(_) => Ok(Msg::Event(Event::SessionDeleted { uid })),
                            Err(error) => Err(Msg::Event(Event::Error {
                                error,
                                source: Box::new(Event::SessionDeleted { uid }),
                            })),
                        }),
                    )))
                    .unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let next_ctx = Ctx::<Env>::default();
                let profile_effects = if next_ctx.profile().ne(self.profile()) {
                    Effects::msg(Msg::Internal(Internal::ProfileChanged))
                } else {
                    Effects::none().unchanged()
                };
                let library_effects = if next_ctx.library().ne(self.library()) {
                    Effects::msg(Msg::Internal(Internal::LibraryChanged(false)))
                } else {
                    Effects::none().unchanged()
                };
                *self = next_ctx;
                Effects::msg(Msg::Event(Event::UserLoggedOut { uid }))
                    .unchanged()
                    .join(session_effects)
                    .join(profile_effects)
                    .join(library_effects)
            }
            Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon))) => {
                let profile = self.profile_mut();
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
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::msg(Msg::Event(Event::Error {
                        error: CtxError::from("Addon already installed"),
                        source: Box::new(Event::AddonInstalled {
                            uid: profile.uid(),
                            addon: addon.to_owned(),
                        }),
                    }))
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(transport_url))) => {
                let profile = self.profile_mut();
                let addon_position = profile.addons.iter().position(|addon| {
                    addon.transport_url.eq(transport_url) && !addon.flags.protected
                });
                if let Some(addon_position) = addon_position {
                    let addon = profile.addons[addon_position].to_owned();
                    profile.addons.remove(addon_position);
                    Effects::msg(Msg::Event(Event::AddonUninstalled {
                        uid: profile.uid(),
                        addon,
                    }))
                    .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    // TODO Consider return error event if addon is not installed
                    // TODO Consider return error event if addon is protected
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::UpdateSettings(settings))) => {
                let profile = self.profile_mut();
                if profile.settings.ne(settings) {
                    profile.settings = settings.to_owned();
                    Effects::msg(Msg::Event(Event::SettingsUpdated { uid: profile.uid() }))
                        .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                } else {
                    Effects::msg(Msg::Event(Event::SettingsUpdated { uid: profile.uid() }))
                        .unchanged()
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::PushUserToAPI)) => {
                // TODO implement
                Effects::msg(Msg::Event(Event::UserPushedToAPI {
                    uid: self.profile().uid(),
                }))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::PullUserFromAPI)) => {
                // TODO implement
                Effects::msg(Msg::Event(Event::UserPulledFromAPI {
                    uid: self.profile().uid(),
                }))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::PushAddonsToAPI)) => {
                match &self.profile().auth {
                    Some(auth) => Effects::one(Box::new(
                        push_addons_to_api::<Env>(&auth.key, &self.profile().addons).then(
                            enclose!((self.profile().uid() => uid)
                                move |result| match result {
                                    Ok(_) => Ok(Msg::Event(Event::AddonsPushedToAPI { uid })),
                                    Err(error) => Ok(Msg::Event(Event::Error {
                                        error,
                                        source: Box::new(Event::AddonsPushedToAPI { uid }),
                                    })),
                                }
                            ),
                        ),
                    ))
                    .unchanged(),
                    _ => {
                        // TODO Consider return error event for user not logged in
                        Effects::none().unchanged()
                    }
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI)) => {
                match &self.profile().auth {
                    Some(auth) => {
                        Effects::one(Box::new(pull_addons_from_api::<Env>(&auth.key, true).then(
                            enclose!((auth.key.to_owned() => auth_key)
                                move |result| {
                                    Ok(Msg::Internal(Internal::AddonsAPIResult(auth_key, result)))
                                }
                            ),
                        )))
                        .unchanged()
                    }
                    _ => {
                        // Does this code need to be moved ?
                        let next_addons = self
                            .profile()
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
                        let profile = self.profile_mut();
                        if profile.addons.ne(&next_addons) {
                            profile.addons = next_addons;
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                                uid: profile.uid(),
                            }))
                            .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                        } else {
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                                uid: profile.uid(),
                            }))
                            .unchanged()
                        }
                    }
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(meta_item))) => {
                let mut lib_item = LibItem {
                    id: meta_item.id.to_owned(),
                    type_name: meta_item.type_name.to_owned(),
                    name: meta_item.name.to_owned(),
                    poster: meta_item.poster.to_owned(),
                    poster_shape: meta_item.poster_shape.to_owned(),
                    behavior_hints: meta_item.behavior_hints.to_owned(),
                    removed: false,
                    temp: false,
                    ctime: Some(Env::now()),
                    mtime: Env::now(),
                    state: LibItemState::default(),
                };
                if let Some(LibItem { ctime, state, .. }) = self.library().items.get(&meta_item.id)
                {
                    lib_item.state = state.to_owned();
                    if let Some(ctime) = ctime {
                        lib_item.ctime = Some(ctime.to_owned());
                    };
                };
                Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item))).unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::RemoveFromLibrary(id))) => {
                if let Some(lib_item) = self.library().items.get(id) {
                    let mut lib_item = lib_item.to_owned();
                    lib_item.removed = true;
                    Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item))).unchanged()
                } else {
                    // TODO Consider return error event for item not in lib
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::RewindLibraryItem(id))) => {
                if let Some(lib_item) = self.library().items.get(id) {
                    let mut lib_item = lib_item.to_owned();
                    lib_item.state.time_offset = 0;
                    Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item))).unchanged()
                } else {
                    // TODO Consider return error event for item not in lib
                    Effects::none().unchanged()
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI)) => {
                match &self.profile().auth {
                    Some(auth) => Effects::one(Box::new(
                        sync_library_with_api::<Env>(&auth.key, self.library().to_owned()).then(
                            enclose!((auth.key.to_owned() => auth_key) move |result| {
                                Ok(Msg::Internal(Internal::LibrarySyncResult(auth_key, result)))
                            }),
                        ),
                    ))
                    .unchanged(),
                    _ => {
                        // TODO Consider return error event for user not logged in or library still loading
                        Effects::none().unchanged()
                    }
                }
            }
            Msg::Internal(Internal::UpdateLibraryItem(lib_item)) => {
                let mut lib_item = lib_item.to_owned();
                lib_item.mtime = Env::now();
                let uid = self.profile().uid();
                let persist_effect: Effect = Box::new(
                    update_and_persist_library::<Env>(
                        self.library_mut(),
                        LibBucket::new(uid.to_owned(), vec![lib_item.to_owned()]),
                    )
                    .then(enclose!((uid) move |result| match result {
                        Ok(_) => Ok(Msg::Event(Event::LibraryPushedToStorage { uid })),
                        Err(error) => Err(Msg::Event(Event::Error {
                            error,
                            source: Box::new(Event::LibraryPushedToStorage { uid }),
                        })),
                    })),
                );
                let push_effect = self.profile().auth.as_ref().map(move |auth| -> Effect {
                    Box::new(push_library_to_api::<Env>(&auth.key, vec![lib_item]).then(
                        move |result| match result {
                            Ok(_) => Ok(Msg::Event(Event::LibraryPushedToAPI { uid })),
                            Err(error) => Err(Msg::Event(Event::Error {
                                error,
                                source: Box::new(Event::LibraryPushedToAPI { uid }),
                            })),
                        },
                    ))
                });
                let library_effects = if let Some(push_effect) = push_effect {
                    Effects::many(vec![persist_effect, push_effect])
                } else {
                    Effects::one(persist_effect)
                };
                library_effects.join(Effects::msg(Msg::Internal(Internal::LibraryChanged(true))))
            }
            Msg::Internal(Internal::ProfileChanged) => Effects::one(Box::new(
                push_profile_to_storage::<Env>(Some(self.profile())).then(
                    enclose!((self.profile().uid() => uid) move |result| {
                        match result {
                            Ok(_) => Ok(Msg::Event(Event::ProfilePushedToStorage { uid })),
                            Err(error) => Err(Msg::Event(Event::Error {
                                error,
                                source: Box::new(Event::ProfilePushedToStorage { uid }),
                            })),
                        }
                    }),
                ),
            ))
            .unchanged(),
            Msg::Internal(Internal::LibraryChanged(persisted)) if !persisted => {
                let (recent_bucket, other_bucket) = self.library().split_by_recent();
                Effects::one(Box::new(
                    push_library_borrowed_to_storage::<Env>(
                        Some(&recent_bucket),
                        Some(&other_bucket),
                    )
                    .then(
                        enclose!((self.profile().uid() => uid) move |result| match result {
                            Ok(_) => Ok(Msg::Event(Event::LibraryPushedToStorage { uid })),
                            Err(error) => Err(Msg::Event(Event::Error {
                                error,
                                source: Box::new(Event::LibraryPushedToStorage { uid }),
                            })),
                        }),
                    ),
                ))
                .unchanged()
            }
            Msg::Internal(Internal::CtxStorageResult(result)) => match &self {
                Ctx::Loading {
                    request: CtxRequest::Storage,
                    ..
                } => match result.deref() {
                    Ok((profile, recent_bucket, other_bucket)) => {
                        let next_proifle = profile.to_owned().unwrap_or_default();
                        let mut next_library = LibBucket::new(next_proifle.uid(), vec![]);
                        if let Some(recent_bucket) = recent_bucket {
                            next_library.merge(recent_bucket.to_owned())
                        };
                        if let Some(other_bucket) = other_bucket {
                            next_library.merge(other_bucket.to_owned())
                        };
                        let next_ctx = Ctx::Ready {
                            profile: next_proifle,
                            library: next_library,
                            env: PhantomData,
                        };
                        let profile_effects = if next_ctx.profile().ne(self.profile()) {
                            Effects::msg(Msg::Internal(Internal::ProfileChanged))
                        } else {
                            Effects::none().unchanged()
                        };
                        let library_effects = if next_ctx.library().ne(self.library()) {
                            Effects::msg(Msg::Internal(Internal::LibraryChanged(false)))
                        } else {
                            Effects::none().unchanged()
                        };
                        *self = next_ctx;
                        Effects::msg(Msg::Event(Event::CtxPulledFromStorage {
                            uid: self.profile().uid(),
                        }))
                        .unchanged()
                        .join(profile_effects)
                        .join(library_effects)
                    }
                    Err(error) => {
                        *self = self.ready();
                        Effects::msg(Msg::Event(Event::Error {
                            error: error.to_owned(),
                            source: Box::new(Event::CtxPulledFromStorage {
                                uid: self.profile().uid(),
                            }),
                        }))
                        .unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match &self {
                Ctx::Loading {
                    request: CtxRequest::API(loading_auth_request),
                    ..
                } if loading_auth_request.eq(auth_request) => match result {
                    Ok((auth, addons, lib_items)) => {
                        let next_proifle = Profile {
                            auth: Some(auth.to_owned()),
                            addons: addons.to_owned(),
                            settings: Settings::default(),
                        };
                        let next_library = LibBucket::new(next_proifle.uid(), lib_items.to_owned());
                        let next_ctx = Ctx::Ready {
                            profile: next_proifle,
                            library: next_library,
                            env: PhantomData,
                        };
                        let profile_effects = if next_ctx.profile().ne(self.profile()) {
                            Effects::msg(Msg::Internal(Internal::ProfileChanged))
                        } else {
                            Effects::none().unchanged()
                        };
                        let library_effects = if next_ctx.library().ne(self.library()) {
                            Effects::msg(Msg::Internal(Internal::LibraryChanged(false)))
                        } else {
                            Effects::none().unchanged()
                        };
                        *self = next_ctx;
                        Effects::msg(Msg::Event(Event::UserAuthenticated {
                            uid: self.profile().uid(),
                            auth_request: auth_request.to_owned(),
                        }))
                        .unchanged()
                        .join(profile_effects)
                        .join(library_effects)
                    }
                    Err(error) => {
                        *self = self.ready();
                        Effects::msg(Msg::Event(Event::Error {
                            error: error.to_owned(),
                            source: Box::new(Event::UserAuthenticated {
                                uid: self.profile().uid(),
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
                    .profile()
                    .auth
                    .as_ref()
                    .map(|auth| &auth.key)
                    .eq(&Some(auth_key)) =>
            {
                match result {
                    Ok(addons) => {
                        let profile = self.profile_mut();
                        if profile.addons.ne(addons) {
                            profile.addons = addons.to_owned();
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                                uid: profile.uid(),
                            }))
                            .join(Effects::msg(Msg::Internal(Internal::ProfileChanged)))
                        } else {
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI {
                                uid: profile.uid(),
                            }))
                            .unchanged()
                        }
                    }
                    Err(error) => Effects::msg(Msg::Event(Event::Error {
                        error: error.to_owned(),
                        source: Box::new(Event::AddonsPulledFromAPI {
                            uid: self.profile().uid(),
                        }),
                    }))
                    .unchanged(),
                }
            }
            Msg::Internal(Internal::LibrarySyncResult(auth_key, result))
                if self
                    .profile()
                    .auth
                    .as_ref()
                    .map(|auth| &auth.key)
                    .eq(&Some(auth_key)) =>
            {
                let uid = self.profile().uid();
                match result {
                    Ok(items) => Effects::msg(Msg::Event(Event::LibrarySyncedWithAPI {
                        uid: uid.to_owned(),
                    }))
                    .join(Effects::one(Box::new(
                        update_and_persist_library::<Env>(
                            self.library_mut(),
                            LibBucket::new(uid.to_owned(), items.to_owned()),
                        )
                        .then(move |result| match result {
                            Ok(_) => Ok(Msg::Event(Event::LibraryPushedToStorage { uid })),
                            Err(error) => Err(Msg::Event(Event::Error {
                                error,
                                source: Box::new(Event::LibraryPushedToStorage { uid }),
                            })),
                        }),
                    )))
                    .join(Effects::msg(Msg::Internal(Internal::LibraryChanged(true)))),
                    Err(error) => Effects::msg(Msg::Event(Event::Error {
                        error: error.to_owned(),
                        source: Box::new(Event::LibrarySyncedWithAPI { uid }),
                    }))
                    .unchanged(),
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}
