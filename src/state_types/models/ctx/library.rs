use super::error::CtxError;
use super::fetch_api;
use crate::constants::{
    LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_COUNT, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
};
use crate::state_types::Environment;
use crate::types::api::{Auth, AuthKey, DatastoreCmd, DatastoreReqBuilder, SuccessResponse};
use crate::types::{LibBucket, LibItem, LibItemModified, UID};
use derivative::Derivative;
use futures::future::Either;
use futures::{future, Future};
use lazysort::SortedBy;

#[derive(Debug, Clone, PartialEq)]
pub enum LibraryRequest {
    Storage,
    API,
}

#[derive(Derivative, Debug, Clone, PartialEq)]
#[derivative(Default)]
pub enum LibraryLoadable {
    Loading(UID, LibraryRequest),
    #[derivative(Default)]
    Ready(LibBucket),
}

impl LibraryLoadable {
    pub fn get_item(&self, id: &str) -> Option<&LibItem> {
        match &self {
            LibraryLoadable::Ready(bucket) => bucket.items.get(id),
            _ => None,
        }
    }
    pub fn pull_from_storage<Env: Environment + 'static>(
    ) -> impl Future<Item = (Option<LibBucket>, Option<LibBucket>), Error = CtxError> {
        Env::get_storage(LIBRARY_RECENT_STORAGE_KEY)
            .join(Env::get_storage(LIBRARY_STORAGE_KEY))
            .map_err(CtxError::from)
    }
    pub fn push_to_storage<Env: Environment + 'static>(
        recent_bucket: Option<&LibBucket>,
        other_bucket: Option<&LibBucket>,
    ) -> impl Future<Item = ((), ()), Error = CtxError> {
        Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, recent_bucket)
            .join(Env::set_storage(LIBRARY_STORAGE_KEY, other_bucket))
            .map_err(CtxError::from)
    }
    pub fn pull_from_api<Env: Environment + 'static>(
        auth_key: &AuthKey,
        ids: Vec<String>,
        all: bool,
    ) -> impl Future<Item = Vec<LibItem>, Error = CtxError> {
        let request = DatastoreReqBuilder::default()
            .auth_key(auth_key.to_owned())
            .collection(LIBRARY_COLLECTION_NAME.to_owned())
            .with_cmd(DatastoreCmd::Get { ids, all });
        fetch_api::<Env, _, _>(&request)
    }
    pub fn push_to_api<Env: Environment + 'static>(
        auth_key: &AuthKey,
        lib_items: Vec<LibItem>,
    ) -> impl Future<Item = (), Error = CtxError> {
        let request = DatastoreReqBuilder::default()
            .auth_key(auth_key.to_owned())
            .collection(LIBRARY_COLLECTION_NAME.to_owned())
            .with_cmd(DatastoreCmd::Put { changes: lib_items });
        fetch_api::<Env, _, SuccessResponse>(&request).map(|_| ())
    }
    pub fn sync_with_api<Env: Environment + 'static>(
        auth_key: &AuthKey,
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
    pub fn update_and_persist<Env: Environment + 'static>(
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
            .all(move |id| recent_items.iter().any(|item| item.id.eq(id)));
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
}
