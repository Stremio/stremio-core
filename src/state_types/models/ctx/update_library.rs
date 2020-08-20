use super::{fetch_api, CtxError, CtxRequest, CtxStatus, OtherError};
use crate::constants::{
    LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_COUNT, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
};
use crate::state_types::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::state_types::{Effect, Effects, Environment};
use crate::types::api::AuthKey;
use crate::types::api::{DatastoreCmd, DatastoreReq, SuccessResponse};
use crate::types::{LibBucket, LibItem, LibItemModified, LibItemState};
use futures::future::Either;
use futures::Future;
use itertools::Itertools;
use std::collections::HashMap;

pub fn update_library<Env: Environment + 'static>(
    library: &mut LibBucket,
    auth_key: Option<&AuthKey>,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
            let next_library = LibBucket::default();
            if *library != next_library {
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
            Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item)))
                .join(Effects::msg(Msg::Event(Event::LibraryItemAdded {
                    id: meta_preview.id.to_owned(),
                })))
                .unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::RemoveFromLibrary(id))) => match library.items.get(id) {
            Some(lib_item) => {
                let mut lib_item = lib_item.to_owned();
                lib_item.removed = true;
                Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item)))
                    .join(Effects::msg(Msg::Event(Event::LibraryItemRemoved {
                        id: id.to_owned(),
                    })))
                    .unchanged()
            }
            _ => Effects::msg(Msg::Event(Event::Error {
                error: CtxError::from(OtherError::LibraryItemNotFound),
                source: Box::new(Event::LibraryItemRemoved { id: id.to_owned() }),
            }))
            .unchanged(),
        },
        Msg::Action(Action::Ctx(ActionCtx::RewindLibraryItem(id))) => match library.items.get(id) {
            Some(lib_item) => {
                let mut lib_item = lib_item.to_owned();
                lib_item.state.time_offset = 0;
                Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item)))
                    .join(Effects::msg(Msg::Event(Event::LibraryItemRewided {
                        id: id.to_owned(),
                    })))
                    .unchanged()
            }
            _ => Effects::msg(Msg::Event(Event::Error {
                error: CtxError::from(OtherError::LibraryItemNotFound),
                source: Box::new(Event::LibraryItemRewided { id: id.to_owned() }),
            }))
            .unchanged(),
        },
        Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI)) => match auth_key {
            Some(auth_key) => {
                Effects::one(plan_sync_with_api::<Env>(library, auth_key)).unchanged()
            }
            _ => Effects::msg(Msg::Event(Event::Error {
                error: CtxError::from(OtherError::UserNotLoggedIn),
                source: Box::new(Event::LibrarySyncWithAPIPlanned {
                    plan: Default::default(),
                }),
            }))
            .unchanged(),
        },
        Msg::Internal(Internal::UpdateLibraryItem(lib_item)) => {
            let mut lib_item = lib_item.to_owned();
            lib_item.mtime = Env::now();
            let push_to_api_effects = match auth_key {
                Some(auth_key) => Effects::one(push_items_to_api::<Env>(
                    vec![lib_item.to_owned()],
                    auth_key,
                ))
                .unchanged(),
                _ => Effects::none().unchanged(),
            };
            let push_to_storage_effects = Effects::one(update_and_push_items_to_storage::<Env>(
                library,
                vec![lib_item],
            ));
            push_to_api_effects
                .join(push_to_storage_effects)
                .join(Effects::msg(Msg::Internal(Internal::LibraryChanged(true))))
        }
        Msg::Internal(Internal::LibraryChanged(persisted)) if !persisted => {
            Effects::one(push_library_to_storage::<Env>(library)).unchanged()
        }
        Msg::Internal(Internal::CtxStorageResult(result)) => match (status, result) {
            (
                CtxStatus::Loading(CtxRequest::Storage),
                Ok((profile, recent_bucket, other_bucket)),
            ) => {
                let mut next_library =
                    LibBucket::new(profile.as_ref().and_then(|profile| profile.uid()), vec![]);
                if let Some((uid, items)) = recent_bucket {
                    if next_library.uid == *uid {
                        next_library.merge(items.to_owned());
                    };
                };
                if let Some((uid, items)) = other_bucket {
                    if next_library.uid == *uid {
                        next_library.merge(items.to_owned());
                    };
                };
                if *library != next_library {
                    *library = next_library;
                    Effects::msg(Msg::Internal(Internal::LibraryChanged(true)))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (
                CtxStatus::Loading(CtxRequest::API(loading_auth_request)),
                Ok((auth, _, lib_items)),
            ) if loading_auth_request == auth_request => {
                let next_library =
                    LibBucket::new(Some(auth.user.id.to_owned()), lib_items.to_owned());
                if *library != next_library {
                    *library = next_library;
                    Effects::msg(Msg::Internal(Internal::LibraryChanged(false)))
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::LibrarySyncPlanResult(
            DatastoreReq {
                auth_key: loading_auth_key,
                ..
            },
            result,
        )) if Some(loading_auth_key) == auth_key => match result {
            Ok((pull_ids, push_ids)) => {
                let push_items = library
                    .items
                    .iter()
                    .filter(move |(id, _)| push_ids.iter().any(|push_id| push_id == *id))
                    .map(|(_, item)| item)
                    .cloned()
                    .collect();
                Effects::msg(Msg::Event(Event::LibrarySyncWithAPIPlanned {
                    plan: (pull_ids.to_owned(), push_ids.to_owned()),
                }))
                .join(Effects::many(vec![
                    push_items_to_api::<Env>(push_items, loading_auth_key),
                    pull_items_from_api::<Env>(pull_ids.to_owned(), loading_auth_key),
                ]))
                .unchanged()
            }
            Err(error) => Effects::msg(Msg::Event(Event::Error {
                error: error.to_owned(),
                source: Box::new(Event::LibrarySyncWithAPIPlanned {
                    plan: Default::default(),
                }),
            }))
            .unchanged(),
        },
        Msg::Internal(Internal::LibraryPullResult(
            DatastoreReq {
                auth_key: loading_auth_key,
                cmd: DatastoreCmd::Get { ids, .. },
                ..
            },
            result,
        )) if Some(loading_auth_key) == auth_key => match result {
            Ok(items) => Effects::msg(Msg::Event(Event::LibraryItemsPulledFromAPI {
                ids: ids.to_owned(),
            }))
            .join(Effects::one(update_and_push_items_to_storage::<Env>(
                library,
                items.to_owned(),
            )))
            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged(true)))),
            Err(error) => Effects::msg(Msg::Event(Event::Error {
                error: error.to_owned(),
                source: Box::new(Event::LibraryItemsPulledFromAPI {
                    ids: ids.to_owned(),
                }),
            }))
            .unchanged(),
        },
        _ => Effects::none().unchanged(),
    }
}

fn update_and_push_items_to_storage<Env: Environment + 'static>(
    library: &mut LibBucket,
    items: Vec<LibItem>,
) -> Effect {
    let ids = items
        .iter()
        .map(|item| &item.id)
        .cloned()
        .collect::<Vec<_>>();
    let are_items_in_recent = library.are_ids_in_recent(&ids);
    library.merge(items);
    let push_to_storage_future = if library.items.len() <= LIBRARY_RECENT_COUNT {
        Either::A(
            Env::set_storage(
                LIBRARY_RECENT_STORAGE_KEY,
                Some(&(&library.uid, library.items.values().collect::<Vec<_>>())),
            )
            .join(Env::set_storage::<()>(LIBRARY_STORAGE_KEY, None))
            .map(|_| ()),
        )
    } else {
        let (recent_items, other_items) = library.split_items_by_recent();
        if are_items_in_recent {
            Either::B(Either::A(Env::set_storage(
                LIBRARY_RECENT_STORAGE_KEY,
                Some(&(&library.uid, recent_items)),
            )))
        } else {
            Either::B(Either::B(
                Env::set_storage(
                    LIBRARY_RECENT_STORAGE_KEY,
                    Some(&(&library.uid, recent_items)),
                )
                .join(Env::set_storage(
                    LIBRARY_STORAGE_KEY,
                    Some(&(&library.uid, other_items)),
                ))
                .map(|_| ()),
            ))
        }
    };
    Box::new(push_to_storage_future.then(move |result| match result {
        Ok(_) => Ok(Msg::Event(Event::LibraryItemsPushedToStorage { ids })),
        Err(error) => Err(Msg::Event(Event::Error {
            error: CtxError::from(error),
            source: Box::new(Event::LibraryItemsPushedToStorage { ids }),
        })),
    }))
}

fn push_library_to_storage<Env: Environment + 'static>(library: &LibBucket) -> Effect {
    let ids = library.items.keys().cloned().collect();
    let (recent_items, other_items) = library.split_items_by_recent();
    Box::new(
        Env::set_storage(
            LIBRARY_RECENT_STORAGE_KEY,
            Some(&(&library.uid, recent_items)),
        )
        .join(Env::set_storage(
            LIBRARY_STORAGE_KEY,
            Some(&(&library.uid, other_items)),
        ))
        .then(move |result| match result {
            Ok(_) => Ok(Msg::Event(Event::LibraryItemsPushedToStorage { ids })),
            Err(error) => Err(Msg::Event(Event::Error {
                error: CtxError::from(error),
                source: Box::new(Event::LibraryItemsPushedToStorage { ids }),
            })),
        }),
    )
}

fn push_items_to_api<Env: Environment + 'static>(items: Vec<LibItem>, auth_key: &str) -> Effect {
    let ids = items.iter().map(|item| &item.id).cloned().collect();
    Box::new(
        fetch_api::<Env, _, SuccessResponse>(&DatastoreReq {
            auth_key: auth_key.to_owned(),
            collection: LIBRARY_COLLECTION_NAME.to_owned(),
            cmd: DatastoreCmd::Put { changes: items },
        })
        .then(move |result| match result {
            Ok(_) => Ok(Msg::Event(Event::LibraryItemsPushedToAPI { ids })),
            Err(error) => Err(Msg::Event(Event::Error {
                error,
                source: Box::new(Event::LibraryItemsPushedToAPI { ids }),
            })),
        }),
    )
}

fn pull_items_from_api<Env: Environment + 'static>(ids: Vec<String>, auth_key: &str) -> Effect {
    let request = DatastoreReq {
        auth_key: auth_key.to_owned(),
        collection: LIBRARY_COLLECTION_NAME.to_owned(),
        cmd: DatastoreCmd::Get { ids, all: false },
    };
    Box::new(
        fetch_api::<Env, _, _>(&request)
            .then(move |result| Ok(Msg::Internal(Internal::LibraryPullResult(request, result)))),
    )
}

fn plan_sync_with_api<Env: Environment + 'static>(library: &LibBucket, auth_key: &str) -> Effect {
    let local_mtimes = library
        .items
        .iter()
        .filter(|(_, item)| item.should_sync())
        .map(|(id, item)| (id.to_owned(), item.mtime.to_owned()))
        .collect::<HashMap<_, _>>();
    let request = DatastoreReq {
        auth_key: auth_key.to_owned(),
        collection: LIBRARY_COLLECTION_NAME.to_owned(),
        cmd: DatastoreCmd::Meta {},
    };
    Box::new(
        fetch_api::<Env, _, Vec<LibItemModified>>(&request)
            .map(|remote_mtimes| {
                remote_mtimes
                    .into_iter()
                    .map(|LibItemModified(id, mtime)| (id, mtime))
                    .collect::<HashMap<_, _>>()
            })
            .map(move |remote_mtimes| {
                let pull_ids = remote_mtimes
                    .iter()
                    .filter(|(id, remote_mtime)| {
                        local_mtimes
                            .get(*id)
                            .map_or(true, |local_mtime| local_mtime < remote_mtime)
                    })
                    .sorted_by(|a, b| a.cmp(b))
                    .map(|(id, _)| id)
                    .cloned()
                    .collect();
                let push_ids = local_mtimes
                    .iter()
                    .filter(|(id, local_mtime)| {
                        remote_mtimes
                            .get(*id)
                            .map_or(true, |remote_mtime| remote_mtime < local_mtime)
                    })
                    .sorted_by(|a, b| a.cmp(b))
                    .map(|(id, _)| id)
                    .cloned()
                    .collect();
                (pull_ids, push_ids)
            })
            .then(move |result| {
                Ok(Msg::Internal(Internal::LibrarySyncPlanResult(
                    request, result,
                )))
            }),
    )
}
