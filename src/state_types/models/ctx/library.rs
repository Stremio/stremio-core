use super::UserDataLoadable;
use crate::constants::{
    LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_COUNT, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
};
use crate::state_types::models::common::{fetch_api, ModelError};
use crate::state_types::msg::{Action, ActionCtx, ActionLibrary, Event, Internal, Msg};
use crate::state_types::{Effect, Effects, Environment};
use crate::types::api::{Auth, DatastoreCmd, DatastoreReqBuilder, SuccessResponse};
use crate::types::{LibBucket, LibItem, LibItemModified, LibItemState, UID};
use chrono::Datelike;
use derivative::Derivative;
use futures::future::Either;
use futures::{future, Future};
use lazysort::SortedBy;

#[derive(Debug, Clone)]
pub enum LibraryRequest {
    Storage,
    API,
}

#[derive(Derivative, Debug, Clone)]
#[derivative(Default)]
pub enum LibraryLoadable {
    Loading(UID, LibraryRequest),
    #[derivative(Default)]
    Ready(LibBucket),
}

impl LibraryLoadable {
    pub fn update<Env: Environment + 'static>(
        &mut self,
        user_data: &UserDataLoadable,
        msg: &Msg,
    ) -> Effects {
        let library_effects = match msg {
            Msg::Action(Action::Ctx(ActionCtx::Library(ActionLibrary::Add(meta_item)))) => {
                let mut lib_item = LibItem {
                    id: meta_item.id.to_owned(),
                    type_name: meta_item.type_name.to_owned(),
                    name: meta_item.name.to_owned(),
                    poster: meta_item.poster.to_owned(),
                    poster_shape: meta_item.poster_shape.to_owned(),
                    logo: meta_item.logo.to_owned(),
                    background: None,
                    year: if let Some(released) = &meta_item.released {
                        Some(released.year().to_string())
                    } else if let Some(release_info) = &meta_item.release_info {
                        Some(release_info.to_owned())
                    } else {
                        None
                    },
                    ctime: Some(Env::now()),
                    mtime: Env::now(),
                    removed: false,
                    temp: false,
                    state: LibItemState::default(),
                };
                if let Some(LibItem { ctime, state, .. }) = self.get_item(&meta_item.id) {
                    lib_item.state = state.to_owned();
                    if let Some(ctime) = ctime {
                        lib_item.ctime = Some(ctime.to_owned());
                    };
                };
                self.set_item::<Env>(user_data.auth(), lib_item)
            }
            Msg::Action(Action::Ctx(ActionCtx::Library(ActionLibrary::Remove(id)))) => {
                match &self {
                    LibraryLoadable::Ready(bucket) => {
                        if let Some(lib_item) = bucket.items.get(id) {
                            let mut lib_item = lib_item.to_owned();
                            lib_item.mtime = Env::now();
                            lib_item.removed = true;
                            self.set_item::<Env>(user_data.auth(), lib_item)
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::Library(ActionLibrary::SyncWithAPI))) => {
                match (user_data.auth(), &self) {
                    (Some(auth), LibraryLoadable::Ready(bucket)) => {
                        let uid = bucket.uid.to_owned();
                        Effects::one(Box::new(lib_sync::<Env>(auth, bucket.to_owned()).then(
                            move |result| {
                                Ok(Msg::Internal(Internal::LibrarySyncResult(uid, result)))
                            },
                        )))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::UpdateLibraryItem(lib_item)) => {
                let mut lib_item = lib_item.to_owned();
                lib_item.mtime = Env::now();
                self.set_item::<Env>(user_data.auth(), lib_item)
            }
            Msg::Internal(Internal::LibraryStorageResult(result)) => match &self {
                LibraryLoadable::Loading(uid, LibraryRequest::Storage) => {
                    let mut bucket = LibBucket::new(uid.to_owned(), vec![]);
                    let bucket_effects = match result {
                        Ok((recent_bucket, other_bucket)) => {
                            if let Some(recent_bucket) = recent_bucket {
                                bucket.merge(recent_bucket.to_owned())
                            };
                            if let Some(other_bucket) = other_bucket {
                                bucket.merge(other_bucket.to_owned())
                            };
                            Effects::msg(Msg::Event(Event::LibraryRetrievedFromStorage))
                        }
                        Err(error) => Effects::msg(Msg::Event(Event::Error(error.to_owned()))),
                    };
                    *self = LibraryLoadable::Ready(bucket);
                    bucket_effects
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::LibraryAPIResult(uid, result)) => match &self {
                LibraryLoadable::Loading(loading_uid, LibraryRequest::API)
                    if loading_uid.eq(&uid) =>
                {
                    let mut bucket = LibBucket::new(uid.to_owned(), vec![]);
                    let bucket_effects = match result {
                        Ok(items) => {
                            bucket.merge(LibBucket::new(uid.to_owned(), items.to_owned()));
                            Effects::msg(Msg::Event(Event::LibrarySyncedWithAPI))
                        }
                        Err(error) => Effects::msg(Msg::Event(Event::Error(error.to_owned()))),
                    };
                    *self = LibraryLoadable::Ready(bucket);
                    bucket_effects
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::LibrarySyncResult(uid, result)) => match self {
                LibraryLoadable::Ready(ref mut bucket) if bucket.uid.eq(uid) => match result {
                    Ok(items) => Effects::msg(Msg::Event(Event::LibrarySyncedWithAPI)).join(
                        Effects::one(Box::new(
                            update_and_persist::<Env>(
                                bucket,
                                LibBucket::new(uid.to_owned(), items.to_owned()),
                            )
                            .map(|_| Msg::Event(Event::LibraryPersisted))
                            .map_err(|error| Msg::Event(Event::Error(error))),
                        )),
                    ),
                    Err(error) => {
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        };
        if library_effects.has_changed {
            Effects::msg(Msg::Internal(Internal::LibraryChanged)).join(library_effects)
        } else {
            library_effects
        }
    }
    pub fn get_item(&self, id: &str) -> Option<&LibItem> {
        match &self {
            LibraryLoadable::Ready(bucket) => bucket.items.get(id),
            _ => None,
        }
    }
    fn set_item<Env: Environment + 'static>(
        &mut self,
        auth: &Option<Auth>,
        lib_item: LibItem,
    ) -> Effects {
        match self {
            LibraryLoadable::Ready(bucket) => {
                let push_effect = auth.as_ref().map(|auth| -> Effect {
                    Box::new(
                        lib_push::<Env>(auth, &lib_item)
                            .map(|_| Msg::Event(Event::LibraryPushedToAPI))
                            .map_err(|error| Msg::Event(Event::Error(error))),
                    )
                });
                let persist_effect: Effect = Box::new(
                    update_and_persist::<Env>(
                        bucket,
                        LibBucket::new(bucket.uid.to_owned(), vec![lib_item]),
                    )
                    .map(|_| Msg::Event(Event::LibraryPersisted))
                    .map_err(|error| Msg::Event(Event::Error(error))),
                );
                if let Some(push_effect) = push_effect {
                    Effects::many(vec![persist_effect, push_effect])
                } else {
                    Effects::one(persist_effect)
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}

// TODO: refactor move lib_sync/lib_pull/lib_push to common crate

fn datastore_req_builder(auth: &Auth) -> DatastoreReqBuilder {
    DatastoreReqBuilder::default()
        .auth_key(auth.key.to_owned())
        .collection(LIBRARY_COLLECTION_NAME.to_owned())
        .clone()
}

fn lib_sync<Env: Environment + 'static>(
    auth: &Auth,
    local_lib: LibBucket,
) -> impl Future<Item = Vec<LibItem>, Error = ModelError> {
    // @TODO consider asserting if uid matches auth
    let builder = datastore_req_builder(auth);
    let meta_req = builder.clone().with_cmd(DatastoreCmd::Meta {});

    fetch_api::<Env, _, Vec<LibItemModified>>(&meta_req).and_then(move |remote_mtimes| {
        let map_remote = remote_mtimes
            .into_iter()
            .map(|LibItemModified(k, mtime)| (k, mtime))
            .collect::<std::collections::HashMap<_, _>>();
        // IDs to pull
        let ids = map_remote
            .iter()
            .filter(|(k, v)| {
                local_lib
                    .items
                    .get(*k)
                    .map_or(true, |item| item.mtime < **v)
            })
            .map(|(k, _)| k.clone())
            .collect::<Vec<String>>();
        // Items to push
        let LibBucket { items, .. } = local_lib;
        let changes = items
            .into_iter()
            .filter(|(id, item)| {
                map_remote.get(id).map_or(true, |date| *date < item.mtime) && item.should_push()
            })
            .map(|(_, v)| v)
            .collect::<Vec<LibItem>>();

        let get_fut = if ids.is_empty() {
            Either::A(future::ok(vec![]))
        } else {
            Either::B(fetch_api::<Env, _, Vec<LibItem>>(
                &builder
                    .clone()
                    .with_cmd(DatastoreCmd::Get { ids, all: false }),
            ))
        };

        let put_fut = if changes.is_empty() {
            Either::A(future::ok(()))
        } else {
            Either::B(
                fetch_api::<Env, _, SuccessResponse>(
                    &builder.clone().with_cmd(DatastoreCmd::Put { changes }),
                )
                .map(|_| ()),
            )
        };

        get_fut.join(put_fut).map(move |(items, _)| items)
    })
}

fn lib_push<Env: Environment + 'static>(
    auth: &Auth,
    item: &LibItem,
) -> impl Future<Item = (), Error = ModelError> {
    let push_req = datastore_req_builder(auth).with_cmd(DatastoreCmd::Put {
        changes: vec![item.to_owned()],
    });

    fetch_api::<Env, _, SuccessResponse>(&push_req).map(|_| ())
}

pub fn lib_pull<Env: Environment + 'static>(
    auth: &Auth,
) -> impl Future<Item = Vec<LibItem>, Error = ModelError> {
    let request = datastore_req_builder(auth).with_cmd(DatastoreCmd::Get {
        ids: vec![],
        all: true,
    });
    fetch_api::<Env, _, _>(&request)
}

fn update_and_persist<Env: Environment + 'static>(
    bucket: &mut LibBucket,
    new_bucket: LibBucket,
) -> impl Future<Item = (), Error = ModelError> {
    let recent_items = bucket
        .items
        .values()
        .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
        .take(LIBRARY_RECENT_COUNT)
        .collect::<Vec<_>>();
    let are_new_items_in_recent = new_bucket
        .items
        .keys()
        .all(move |id| recent_items.iter().any(|item| item.id.eq(id)));
    bucket.merge(new_bucket);
    if bucket.items.len() <= LIBRARY_RECENT_COUNT {
        Either::A(
            Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(bucket))
                .join(Env::set_storage::<LibBucket>(LIBRARY_STORAGE_KEY, None))
                .map(|(_, _)| ())
                .map_err(ModelError::from),
        )
    } else {
        let (recent_bucket, other_bucket) = bucket.split_by_recent();
        if are_new_items_in_recent {
            Either::B(Either::A(
                Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(&recent_bucket))
                    .map_err(ModelError::from),
            ))
        } else {
            Either::B(Either::B(
                Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(&recent_bucket))
                    .join(Env::set_storage(LIBRARY_STORAGE_KEY, Some(&other_bucket)))
                    .map(|(_, _)| ())
                    .map_err(ModelError::from),
            ))
        }
    }
}
