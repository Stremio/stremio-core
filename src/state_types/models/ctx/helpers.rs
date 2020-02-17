use super::CtxError;
use crate::constants::{
    LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_COUNT, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
    PROFILE_STORAGE_KEY,
};
use crate::state_types::{Environment, Request};
use crate::types::addons::Descriptor;
use crate::types::api::{
    APIMethodName, APIRequest, APIResult, Auth, AuthRequest, AuthResponse, CollectionResponse,
    DatastoreCmd, DatastoreReqBuilder, SuccessResponse,
};
use crate::types::profile::Profile;
use crate::types::{LibBucket, LibBucketBorrowed, LibItem, LibItemModified};
use futures::future::Either;
use futures::{future, Future};
use lazysort::SortedBy;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn pull_profile_from_storage<Env: Environment + 'static>(
) -> impl Future<Item = Option<Profile>, Error = CtxError> {
    Env::get_storage(PROFILE_STORAGE_KEY).map_err(CtxError::from)
}

pub fn push_profile_to_storage<Env: Environment + 'static>(
    profile: Option<&Profile>,
) -> impl Future<Item = (), Error = CtxError> {
    Env::set_storage(PROFILE_STORAGE_KEY, profile).map_err(CtxError::from)
}

pub fn authenticate<Env: Environment + 'static>(
    request: &AuthRequest,
) -> impl Future<Item = Auth, Error = CtxError> {
    fetch_api::<Env, _, _>(&APIRequest::Auth(request.to_owned()))
        .map(|AuthResponse { key, user }| Auth { key, user })
}

pub fn delete_session<Env: Environment + 'static>(
    auth_key: &str,
) -> impl Future<Item = (), Error = CtxError> {
    fetch_api::<Env, _, SuccessResponse>(&APIRequest::Logout {
        auth_key: auth_key.to_owned(),
    })
    .map(|_| ())
}

pub fn pull_user_from_api<Env: Environment + 'static>(_auth_key: &str) {
    unimplemented!();
}

pub fn push_user_to_api<Env: Environment + 'static>(_auth_key: &str) {
    unimplemented!();
}

pub fn pull_addons_from_api<Env: Environment + 'static>(
    auth_key: &str,
    update: bool,
) -> impl Future<Item = Vec<Descriptor>, Error = CtxError> {
    fetch_api::<Env, _, _>(&APIRequest::AddonCollectionGet {
        auth_key: auth_key.to_owned(),
        update,
    })
    .map(|CollectionResponse { addons, .. }| addons)
}

pub fn push_addons_to_api<Env: Environment + 'static>(
    auth_key: &str,
    addons: &[Descriptor],
) -> impl Future<Item = (), Error = CtxError> {
    fetch_api::<Env, _, SuccessResponse>(&APIRequest::AddonCollectionSet {
        auth_key: auth_key.to_owned(),
        addons: addons.to_owned(),
    })
    .map(|_| ())
}

pub fn pull_library_from_storage<Env: Environment + 'static>(
) -> impl Future<Item = (Option<LibBucket>, Option<LibBucket>), Error = CtxError> {
    Env::get_storage(LIBRARY_RECENT_STORAGE_KEY)
        .join(Env::get_storage(LIBRARY_STORAGE_KEY))
        .map_err(CtxError::from)
}

pub fn push_library_to_storage<Env: Environment + 'static>(
    recent_bucket: Option<&LibBucket>,
    other_bucket: Option<&LibBucket>,
) -> impl Future<Item = ((), ()), Error = CtxError> {
    Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, recent_bucket)
        .join(Env::set_storage(LIBRARY_STORAGE_KEY, other_bucket))
        .map_err(CtxError::from)
}

pub fn push_library_borrowed_to_storage<Env: Environment + 'static>(
    recent_bucket: Option<&LibBucketBorrowed>,
    other_bucket: Option<&LibBucketBorrowed>,
) -> impl Future<Item = ((), ()), Error = CtxError> {
    Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, recent_bucket)
        .join(Env::set_storage(LIBRARY_STORAGE_KEY, other_bucket))
        .map_err(CtxError::from)
}

pub fn pull_library_from_api<Env: Environment + 'static>(
    auth_key: &str,
    ids: Vec<String>,
    all: bool,
) -> impl Future<Item = Vec<LibItem>, Error = CtxError> {
    let request = DatastoreReqBuilder::default()
        .auth_key(auth_key.to_owned())
        .collection(LIBRARY_COLLECTION_NAME.to_owned())
        .with_cmd(DatastoreCmd::Get { ids, all });
    fetch_api::<Env, _, _>(&request)
}

pub fn push_library_to_api<Env: Environment + 'static>(
    auth_key: &str,
    lib_items: Vec<LibItem>,
) -> impl Future<Item = (), Error = CtxError> {
    let request = DatastoreReqBuilder::default()
        .auth_key(auth_key.to_owned())
        .collection(LIBRARY_COLLECTION_NAME.to_owned())
        .with_cmd(DatastoreCmd::Put { changes: lib_items });
    fetch_api::<Env, _, SuccessResponse>(&request).map(|_| ())
}

pub fn sync_library_with_api<Env: Environment + 'static>(
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

pub fn update_and_persist_library<Env: Environment + 'static>(
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

fn fetch_api<Env, REQ, RESP>(api_request: &REQ) -> impl Future<Item = RESP, Error = CtxError>
where
    Env: Environment + 'static,
    REQ: APIMethodName + Clone + Serialize + 'static,
    RESP: DeserializeOwned + 'static,
{
    let url = format!("{}/api/{}", Env::api_url(), api_request.method_name());
    let request = Request::post(url)
        .body(api_request.to_owned())
        .expect("fetch_api request builder cannot fail");
    Env::fetch_serde::<_, _>(request)
        .map_err(CtxError::from)
        .and_then(|result| match result {
            APIResult::Ok { result } => Ok(result),
            APIResult::Err { error } => Err(CtxError::from(error)),
        })
}
