use crate::state_types::messages::Event::*;
use crate::state_types::messages::Internal::*;
use crate::state_types::messages::*;
use crate::state_types::models::*;
use crate::state_types::*;
use crate::types::api::*;
use crate::types::{LibBucket, LibItem, LibItemModified, LibItemState, LIB_RECENT_COUNT, UID};
use chrono::Datelike;
use derivative::*;
use enclose::*;
use futures::future::Either;
use futures::{future, Future};
use lazysort::SortedBy;

const COLL_NAME: &str = "libraryItem";
const STORAGE_RECENT_SLOT: &str = "recent_library";
const STORAGE_SLOT: &str = "library";

#[derive(Derivative, PartialEq)]
#[derivative(Debug, Default, Clone)]
pub enum LibraryLoadable {
    // NotLoaded: we've never attempted loading the library index
    #[derivative(Default)]
    NotLoaded,
    Loading(UID),
    Ready(LibBucket),
}

impl LibraryLoadable {
    pub fn get(&self, id: &str) -> Option<&LibItem> {
        match self {
            LibraryLoadable::Ready(bucket) => bucket.items.get(id),
            _ => None,
        }
    }
    pub fn load_from_storage<Env: Environment + 'static>(
        &mut self,
        content: &CtxContent,
    ) -> Effects {
        let uid: UID = content.auth.as_ref().into();
        *self = LibraryLoadable::Loading(uid.to_owned());

        let mut default_bucket = LibBucket::new(uid, vec![]);
        let ft = Env::get_storage::<LibBucket>(STORAGE_SLOT)
            .join(Env::get_storage::<LibBucket>(STORAGE_RECENT_SLOT))
            .map(move |(a, b)| {
                for loaded_bucket in a.into_iter().chain(b.into_iter()) {
                    default_bucket.try_merge(loaded_bucket);
                }
                default_bucket
            })
            .map(|bucket| LibLoaded(bucket).into())
            .map_err(|e| LibFatal(e.into()).into());
        Effects::one(Box::new(ft))
    }
    pub fn load_initial<Env: Environment + 'static>(&mut self, content: &CtxContent) -> Effects {
        *self = match &content.auth {
            None => LibraryLoadable::Ready(Default::default()),
            Some(a) => LibraryLoadable::Loading(Some(a).into()),
        };

        match &content.auth {
            None => Effects::none(),
            Some(a) => {
                let get_req = DatastoreReqBuilder::default()
                    .auth_key(a.key.to_owned())
                    .collection(COLL_NAME.to_owned())
                    .with_cmd(DatastoreCmd::Get {
                        ids: vec![],
                        all: true,
                    });

                let uid: UID = content.auth.as_ref().into();
                let mut bucket = LibBucket::new(uid.clone(), vec![]);
                let ft = api_fetch::<Env, Vec<LibItem>, _>(get_req)
                    .and_then(move |items| {
                        update_and_persist::<Env>(&mut bucket, LibBucket::new(uid, items))
                            .map(move |_| LibLoaded(bucket).into())
                            .map_err(Into::into)
                    })
                    .map_err(|e| LibFatal(e).into());

                Effects::one(Box::new(ft))
            }
        }
    }
    pub fn update<Env: Environment + 'static>(
        &mut self,
        content: &CtxContent,
        msg: &Msg,
    ) -> Effects {
        match self {
            LibraryLoadable::Loading(uid) => match msg {
                Msg::Internal(LibLoaded(bucket)) if &bucket.uid == uid => {
                    *self = LibraryLoadable::Ready(bucket.clone());
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            },
            LibraryLoadable::Ready(ref mut lib_bucket) => {
                match msg {
                    // User actions
                    Msg::Action(Action::UserOp(action)) => {
                        let err_mapper = enclose!((action) move |e| CtxActionErr(action, e).into());
                        match action {
                            ActionUser::LibSync => {
                                if let Some(auth) = &content.auth {
                                    let ft = lib_sync::<Env>(auth, lib_bucket.clone())
                                        .map(|bucket| LibSyncPulled(bucket).into())
                                        .map_err(err_mapper);
                                    Effects::one(Box::new(ft)).unchanged()
                                } else {
                                    Effects::none().unchanged()
                                }
                            }
                            ActionUser::AddToLibrary { meta_item, now } => {
                                let mut next_lib_item = LibItem {
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
                                    ctime: Some(now.to_owned()),
                                    mtime: now.to_owned(),
                                    removed: false,
                                    temp: false,
                                    state: LibItemState::default(),
                                };
                                if let Some(lib_item) = lib_bucket.items.get(&meta_item.id) {
                                    next_lib_item.state = lib_item.state.to_owned();
                                    if let Some(ctime) = lib_item.ctime {
                                        next_lib_item.ctime = Some(ctime);
                                    };
                                };
                                self.update::<Env>(
                                    &content,
                                    &Msg::Action(Action::UserOp(ActionUser::LibUpdate(
                                        next_lib_item,
                                    ))),
                                )
                            }
                            ActionUser::RemoveFromLibrary { id, now } => {
                                match lib_bucket.items.get(id) {
                                    Some(lib_item) => {
                                        let mut lib_item = lib_item.to_owned();
                                        lib_item.removed = true;
                                        lib_item.mtime = now.to_owned();
                                        self.update::<Env>(
                                            &content,
                                            &Msg::Action(Action::UserOp(ActionUser::LibUpdate(
                                                lib_item,
                                            ))),
                                        )
                                    }
                                    None => Effects::none().unchanged(),
                                }
                            }
                            ActionUser::LibUpdate(item) => {
                                let new_bucket = LibBucket::new(
                                    content.auth.as_ref().into(),
                                    vec![item.to_owned()],
                                );
                                let persist_ft = update_and_persist::<Env>(lib_bucket, new_bucket)
                                    .map(|_| LibPersisted.into())
                                    .map_err(err_mapper.clone());

                                // If we're logged in, push to API
                                if let Some(auth) = &content.auth {
                                    let push_ft = lib_push::<Env>(auth, &item)
                                        .map(|_| LibPushed.into())
                                        .map_err(err_mapper);
                                    Effects::many(vec![Box::new(persist_ft), Box::new(push_ft)])
                                } else {
                                    Effects::one(Box::new(persist_ft))
                                }
                            }
                            _ => Effects::none().unchanged(),
                        }
                    }
                    Msg::Internal(LibSyncPulled(new_bucket)) if !new_bucket.items.is_empty() => {
                        let ft = update_and_persist::<Env>(lib_bucket, new_bucket.clone())
                            .map(|_| LibPersisted.into())
                            .map_err(move |e| LibFatal(e).into());
                        Effects::one(Box::new(ft))
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            // Ignore NotLoaded state: the load_* methods are supposed to take us out of it
            _ => Effects::none().unchanged(),
        }
    }
}

fn datastore_req_builder(auth: &Auth) -> DatastoreReqBuilder {
    DatastoreReqBuilder::default()
        .auth_key(auth.key.to_owned())
        .collection(COLL_NAME.to_owned())
        .clone()
}

fn lib_sync<Env: Environment + 'static>(
    auth: &Auth,
    local_lib: LibBucket,
) -> impl Future<Item = LibBucket, Error = CtxError> {
    // @TODO consider asserting if uid matches auth
    let builder = datastore_req_builder(auth);
    let meta_req = builder.clone().with_cmd(DatastoreCmd::Meta {});

    api_fetch::<Env, Vec<LibItemModified>, _>(meta_req).and_then(move |remote_mtimes| {
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
        let LibBucket { items, uid } = local_lib;
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
            Either::B(api_fetch::<Env, Vec<LibItem>, _>(
                builder
                    .clone()
                    .with_cmd(DatastoreCmd::Get { ids, all: false }),
            ))
        };

        let put_fut = if changes.is_empty() {
            Either::A(future::ok(()))
        } else {
            Either::B(
                api_fetch::<Env, SuccessResponse, _>(
                    builder.clone().with_cmd(DatastoreCmd::Put { changes }),
                )
                .map(|_| ()),
            )
        };

        get_fut
            .join(put_fut)
            .map(move |(items, _)| LibBucket::new(uid, items))
    })
}

fn lib_push<Env: Environment + 'static>(
    auth: &Auth,
    item: &LibItem,
) -> impl Future<Item = (), Error = CtxError> {
    let push_req = datastore_req_builder(auth).with_cmd(DatastoreCmd::Put {
        changes: vec![item.to_owned()],
    });

    api_fetch::<Env, SuccessResponse, _>(push_req).map(|_| ())
}

fn update_and_persist<Env: Environment + 'static>(
    bucket: &mut LibBucket,
    new_bucket: LibBucket,
) -> impl Future<Item = (), Error = CtxError> {
    // @TODO we should consider reading that from storage, it will have stronger consistency
    // guarantees

    // Determine whether all of the new items are in the current recent
    let new_were_in_recent = new_bucket.items.len() <= bucket.items.len() && {
        let current_recent: Vec<&str> = bucket
            .items
            .values()
            .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
            .take(LIB_RECENT_COUNT)
            .map(|item| item.id.as_str())
            .collect();

        new_bucket
            .items
            .keys()
            .all(move |id| current_recent.iter().any(|x| *x == id))
    };

    // Merge the buckets
    bucket.try_merge(new_bucket);

    // If there are less items than the threshold, we can save everything in the recent slot
    // otherwise, we will only save the recent bucket if all of the modified items were previously
    // in the recent bucket;
    if bucket.items.len() <= LIB_RECENT_COUNT {
        Either::A(Env::set_storage(STORAGE_RECENT_SLOT, Some(bucket)).map_err(Into::into))
    } else {
        let (recent, other) = bucket.split_by_recent();
        if new_were_in_recent {
            Either::A(Env::set_storage(STORAGE_RECENT_SLOT, Some(&recent)).map_err(Into::into))
        } else {
            Either::B(
                Env::set_storage(STORAGE_RECENT_SLOT, Some(&recent))
                    .join(Env::set_storage(STORAGE_SLOT, Some(&other)))
                    .map(|(_, _)| ())
                    .map_err(Into::into),
            )
        }
    }
}
