use super::{fetch_api, CtxError, CtxRequest, CtxStatus};
use crate::constants::{
    LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_COUNT, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
};
use crate::state_types::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::state_types::{Effects, Environment};
use crate::types::api::{DatastoreCmd, DatastoreReq, DatastoreReqBuilder, SuccessResponse};
use crate::types::profile::Profile;
use crate::types::{LibBucket, LibItem, LibItemModified, LibItemState};
use enclose::enclose;
use futures::future::Either;
use futures::{future, Future};
use lazysort::SortedBy;
use std::ops::Deref;

pub fn update_library<Env: Environment + 'static>(
    library: &mut LibBucket,
    profile: &Profile,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
            let next_library = LibBucket::default();
            if next_library.ne(library) {
                *library = next_library;
                Effects::msg(Msg::Internal(Internal::LibraryChanged(false)))
            } else {
                Effects::none().unchanged()
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(meta_preview))) => {
            let mut lib_item = LibItem {
                id: meta_preview.id.to_owned(),
                type_name: meta_preview.type_name.to_owned(),
                name: meta_preview.name.to_owned(),
                poster: meta_preview.poster.to_owned(),
                poster_shape: meta_preview.poster_shape.to_owned(),
                behavior_hints: meta_preview.behavior_hints.to_owned(),
                removed: false,
                temp: false,
                mtime: Env::now(),
                ctime: Some(Env::now()),
                state: LibItemState::default(),
            };
            if let Some(LibItem { ctime, state, .. }) = library.items.get(&meta_preview.id) {
                lib_item.state = state.to_owned();
                if let Some(ctime) = ctime {
                    lib_item.ctime = Some(ctime.to_owned());
                };
            };
            Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item))).unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::RemoveFromLibrary(id))) => {
            match library.items.get(id) {
                Some(lib_item) => {
                    let mut lib_item = lib_item.to_owned();
                    lib_item.removed = true;
                    Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item))).unchanged()
                }
                _ => {
                    // TODO Consider return error event for item not in lib
                    Effects::none().unchanged()
                }
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::RewindLibraryItem(id))) => {
            match library.items.get(id) {
                Some(lib_item) => {
                    let mut lib_item = lib_item.to_owned();
                    lib_item.state.time_offset = 0;
                    Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item))).unchanged()
                }
                _ => {
                    // TODO Consider return error event for item not in lib
                    Effects::none().unchanged()
                }
            }
        }
        Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI)) => {
            match &profile.auth {
                Some(auth) => Effects::one(Box::new(
                    sync_with_api::<Env>(&auth.key, library.to_owned()).then(
                        enclose!((auth.key.to_owned() => auth_key) move |result| {
                            Ok(Msg::Internal(Internal::LibrarySyncResult(auth_key, result)))
                        }),
                    ),
                ))
                .unchanged(),
                _ => {
                    // TODO Consider return error event for user not logged in
                    Effects::none().unchanged()
                }
            }
        }
        Msg::Internal(Internal::UpdateLibraryItem(lib_item)) => {
            let mut lib_item = lib_item.to_owned();
            lib_item.mtime = Env::now();
            let persist_effects = Effects::one(Box::new(
                update_and_persist::<Env>(
                    library,
                    LibBucket::new(profile.uid(), vec![lib_item.to_owned()]),
                )
                .map(enclose!((profile.uid() => uid) move |_| {
                    Msg::Event(Event::LibraryPushedToStorage { uid })
                }))
                .map_err(enclose!((profile.uid() => uid) move |error| {
                    Msg::Event(Event::Error {
                        error,
                        source: Box::new(Event::LibraryPushedToStorage { uid }),
                    })
                })),
            ));
            let push_effects = match &profile.auth {
                Some(auth) => Effects::one(Box::new(
                    fetch_api::<Env, _, SuccessResponse>(&DatastoreReq {
                        auth_key: auth.key.to_owned(),
                        collection: LIBRARY_COLLECTION_NAME.to_owned(),
                        cmd: DatastoreCmd::Put {
                            changes: vec![lib_item],
                        },
                    })
                    .map(enclose!((profile.uid() => uid) move |_| {
                        Msg::Event(Event::LibraryPushedToAPI { uid })
                    }))
                    .map_err(enclose!((profile.uid() => uid) move |error| {
                        Msg::Event(Event::Error {
                            error,
                            source: Box::new(Event::LibraryPushedToAPI { uid }),
                        })
                    })),
                ))
                .unchanged(),
                _ => Effects::none().unchanged(),
            };
            persist_effects
                .join(push_effects)
                .join(Effects::msg(Msg::Internal(Internal::LibraryChanged(true))))
        }
        Msg::Internal(Internal::LibraryChanged(persisted)) if !persisted => {
            let (recent_bucket, other_bucket) = library.split_by_recent();
            Effects::one(Box::new(
                Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(&recent_bucket))
                    .join(Env::set_storage(LIBRARY_STORAGE_KEY, Some(&other_bucket)))
                    .map(enclose!((profile.uid() => uid) move |_| {
                        Msg::Event(Event::LibraryPushedToStorage { uid })
                    }))
                    .map_err(enclose!((profile.uid() => uid) move |error| {
                        Msg::Event(Event::Error {
                            error: CtxError::from(error),
                            source: Box::new(Event::LibraryPushedToStorage { uid }),
                        })
                    })),
            ))
            .unchanged()
        }
        Msg::Internal(Internal::CtxStorageResult(result)) => match (status, result.deref()) {
            (CtxStatus::Loading(CtxRequest::Storage), Ok((_, recent_bucket, other_bucket))) => {
                let mut next_library = LibBucket::new(profile.uid(), vec![]);
                if let Some(recent_bucket) = recent_bucket {
                    next_library.merge(recent_bucket.to_owned())
                };
                if let Some(other_bucket) = other_bucket {
                    next_library.merge(other_bucket.to_owned())
                };
                if next_library.ne(library) {
                    Effects::msg(Msg::Internal(Internal::LibraryChanged(false)))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(CtxRequest::API(loading_auth_request)), Ok((_, _, lib_items)))
                if loading_auth_request == auth_request =>
            {
                let next_library = LibBucket::new(profile.uid(), lib_items.to_owned());
                if next_library.ne(library) {
                    Effects::msg(Msg::Internal(Internal::LibraryChanged(false)))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::LibrarySyncResult(auth_key, result))
            if profile.auth.as_ref().map(|auth| &auth.key) == Some(auth_key) =>
        {
            match result {
                Ok(items) => Effects::msg(Msg::Event(Event::LibrarySyncedWithAPI {
                    uid: profile.uid(),
                }))
                .join(Effects::one(Box::new(
                    update_and_persist::<Env>(
                        library,
                        LibBucket::new(profile.uid(), items.to_owned()),
                    )
                    .map(enclose!((profile.uid() => uid) move |_| {
                        Msg::Event(Event::LibraryPushedToStorage { uid })
                    }))
                    .map_err(enclose!((profile.uid() => uid) move |error| {
                        Msg::Event(Event::Error {
                            error,
                            source: Box::new(Event::LibraryPushedToStorage { uid }),
                        })
                    })),
                )))
                .join(Effects::msg(Msg::Internal(Internal::LibraryChanged(true)))),
                Err(error) => Effects::msg(Msg::Event(Event::Error {
                    error: error.to_owned(),
                    source: Box::new(Event::LibrarySyncedWithAPI { uid: profile.uid() }),
                }))
                .unchanged(),
            }
        }
        _ => Effects::none().unchanged(),
    }
}

// TODO consider use LibBucketBorrowed
fn sync_with_api<Env: Environment + 'static>(
    auth_key: &str,
    local_lib: LibBucket,
) -> impl Future<Item = Vec<LibItem>, Error = CtxError> {
    // @TODO consider asserting if uid matches auth
    let builder = DatastoreReqBuilder::default()
        .auth_key(auth_key.to_owned())
        .collection(LIBRARY_COLLECTION_NAME.to_owned())
        .clone();
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

fn update_and_persist<Env: Environment + 'static>(
    bucket: &mut LibBucket,
    new_bucket: LibBucket,
) -> impl Future<Item = (), Error = CtxError> {
    let recent_items = bucket
        .items
        .values()
        .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
        .take(LIBRARY_RECENT_COUNT)
        .collect::<Vec<_>>();
    let are_new_items_in_recent = new_bucket
        .items
        .keys()
        .all(move |id| recent_items.iter().any(|item| item.id == *id));
    bucket.merge(new_bucket);
    if bucket.items.len() <= LIBRARY_RECENT_COUNT {
        Either::A(
            Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(bucket))
                .join(Env::set_storage::<LibBucket>(LIBRARY_STORAGE_KEY, None))
                .map(|(_, _)| ())
                .map_err(CtxError::from),
        )
    } else {
        let (recent_bucket, other_bucket) = bucket.split_by_recent();
        if are_new_items_in_recent {
            Either::B(Either::A(
                Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(&recent_bucket))
                    .map_err(CtxError::from),
            ))
        } else {
            Either::B(Either::B(
                Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(&recent_bucket))
                    .join(Env::set_storage(LIBRARY_STORAGE_KEY, Some(&other_bucket)))
                    .map(|(_, _)| ())
                    .map_err(CtxError::from),
            ))
        }
    }
}
