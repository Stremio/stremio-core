use crate::state_types::Event::*;
use crate::state_types::Internal::*;
use crate::state_types::*;
use crate::types::api::*;
use crate::types::{LibItem, LibItemModified};
use derivative::*;
use enclose::*;
use futures::future::Either;
use futures::{future, Future};
use serde_derive::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use lazysort::SortedBy;

const COLL_NAME: &str = "libraryItem";
const STORAGE_RECENT: &str = "recent_library";
const STORAGE_SLOT: &str = "library";

// According to a mid-2019 study, only 2.7% of users
// have a library larger than that
const RECENT_COUNT: usize = 200;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct UID(Option<String>);
impl From<Option<&Auth>> for UID {
    fn from(a: Option<&Auth>) -> Self {
        UID(a.map(|a| a.user.id.to_owned()))
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct LibBucket {
    pub uid: UID,
    pub items: HashMap<String, LibItem>,
}
impl LibBucket {
    fn new(uid: UID, items: Vec<LibItem>) -> Self {
        LibBucket {
            uid,
            items: items.into_iter().map(|i| (i.id.to_owned(), i)).collect(),
        }
    }
    fn try_merge(&mut self, other: LibBucket) -> &Self {
        if self.uid != other.uid {
            return self;
        }

        for (k, item) in other.items.into_iter() {
            match self.items.entry(k) {
                Vacant(entry) => {
                    entry.insert(item);
                }
                Occupied(mut entry) => {
                    if item.mtime > entry.get().mtime {
                        entry.insert(item);
                    }
                }
            }
        }

        self
    }
}

#[derive(Derivative)]
#[derivative(Debug, Default, Clone)]
pub enum LibraryLoadable {
    // NotLoaded: we've never attempted loading the library index
    #[derivative(Default)]
    NotLoaded,
    Loading(UID),
    Ready(LibBucket),
}

impl LibraryLoadable {
    pub fn load_from_storage<Env: Environment + 'static>(
        &mut self,
        content: &CtxContent,
    ) -> Effects {
        let uid: UID = content.auth.as_ref().into();
        *self = LibraryLoadable::Loading(uid.to_owned());

        let mut default_bucket = LibBucket::new(uid, vec![]);
        let ft = Env::get_storage::<LibBucket>(STORAGE_SLOT)
            .join(Env::get_storage::<LibBucket>(STORAGE_RECENT))
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
                        let auth = match &content.auth {
                            Some(auth) => auth,
                            None => return Effects::none().unchanged(),
                        };
                        let err_mapper = enclose!((action) move |e| CtxActionErr(action, e).into());
                        match action {
                            ActionUser::LibSync => {
                                let ft = lib_sync::<Env>(auth, lib_bucket.clone())
                                    .map(|bucket| LibSyncPulled(bucket).into())
                                    .map_err(err_mapper);
                                Effects::one(Box::new(ft)).unchanged()
                            }
                            ActionUser::LibUpdate(item) => {
                                // @TODO do we really need to use a bucket here?
                                let new_bucket = LibBucket::new(
                                    content.auth.as_ref().into(),
                                    vec![item.clone()],
                                );
                                let persist_ft = update_and_persist::<Env>(lib_bucket, new_bucket)
                                    .map(|_| LibPersisted.into())
                                    .map_err(err_mapper.clone());
                                let push_ft = lib_push::<Env>(auth, &item)
                                    .map(|_| LibPushed.into())
                                    .map_err(err_mapper);
                                Effects::many(vec![Box::new(persist_ft), Box::new(push_ft)])
                            }
                            _ => Effects::none().unchanged(),
                        }
                    }
                    Msg::Internal(LibSyncPulled(new_bucket)) => {
                        // @TODO: can we get rid of this clone?
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
            .collect::<HashMap<_, _>>();
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

        let get_fut = if ids.len() > 0 {
            Either::A(api_fetch::<Env, Vec<LibItem>, _>(
                builder
                    .clone()
                    .with_cmd(DatastoreCmd::Get { ids, all: false }),
            ))
        } else {
            Either::B(future::ok(vec![]))
        };

        let put_fut = if changes.len() > 0 {
            Either::A(
                api_fetch::<Env, SuccessResponse, _>(
                    builder.clone().with_cmd(DatastoreCmd::Put { changes }),
                )
                .map(|_| ()),
            )
        } else {
            Either::B(future::ok(()))
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
    let current_recent: Vec<&str> = bucket
        .items
        .values()
        .map(|item| (item.id.as_str(), &item.mtime))
        .sorted_by(|a, b| b.1.cmp(&a.1))
        .take(RECENT_COUNT)
        .map(|(id, _)| id)
        .collect();
    let all_in_recent = new_bucket
        .items
        .keys()
        .all(move |id| current_recent.iter().any(|x| *x == id));

    // Merge the buckets
    bucket.try_merge(new_bucket);

    // @TODO explain this
    if bucket.items.len() <= RECENT_COUNT {
        // only save one bucket
    } else if all_in_recent {
        // save one bucket, but only the newest
        // @TODO borrowed bucket or clone
    } else {
        // @TODO borrowed bucketS or clone
    }

    Env::set_storage(STORAGE_SLOT, Some(bucket)).map_err(Into::into)
}
