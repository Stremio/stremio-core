use crate::state_types::Event::*;
use crate::state_types::Internal::*;
use crate::state_types::*;
use crate::types::api::*;
use crate::types::LibItem;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use derivative::*;
use enclose::*;
use futures::future::Either;
use futures::{future, Future};
use serde_derive::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

const COLL_NAME: &str = "libraryItem";
const STORAGE_RECENT: &str = "recent_library";
const STORAGE_SLOT: &str = "library";
const RECENT_COUNT: usize = 40;

type UID = Option<String>;

#[derive(Debug, Deserialize)]
struct LibMTime(String, #[serde(with = "ts_milliseconds")] DateTime<Utc>);

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct LibBucket {
    pub uid: UID,
    pub items: HashMap<String, LibItem>,
}
impl LibBucket {
    fn new(auth: &Option<Auth>, items: Vec<LibItem>) -> Self {
        LibBucket {
            uid: auth.as_ref().map(|a| a.user.id.to_owned()),
            items: items
                .iter()
                .map(|i| (i.id.to_owned(), i.to_owned()))
                .collect(),
        }
    }
    fn try_merge(mut self, other: LibBucket) -> Self {
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
                _ => (),
            }
        }

        self
    }
    fn try_merge_opt(mut self, other: Option<LibBucket>) -> Self {
        match other {
            Some(other) => self.try_merge(other),
            None => self,
        }
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
        let uid = content.auth.as_ref().map(|a| a.user.id.to_owned());
        *self = LibraryLoadable::Loading(uid.to_owned());

        let initial_bucket = LibBucket {
            uid,
            ..Default::default()
        };
        let ft = Env::get_storage::<LibBucket>(COLL_NAME)
            .map(move |b| LibLoaded(initial_bucket.try_merge_opt(b)).into())
            .map_err(|e| LibFatal(e.into()).into());
        Effects::one(Box::new(ft))
    }
    pub fn load_initial_api<Env: Environment + 'static>(
        &mut self,
        content: &CtxContent,
    ) -> Effects {
        *self = match &content.auth {
            None => LibraryLoadable::Ready(Default::default()),
            Some(a) => LibraryLoadable::Loading(Some(a.user.id.to_owned())),
        };

        match &content.auth {
            None => Effects::none(),
            Some(a) => {
                let key = &a.key;
                let get_req = DatastoreReqBuilder::default()
                    .auth_key(key.to_owned())
                    .collection(COLL_NAME.to_owned())
                    .with_cmd(DatastoreCmd::Get {
                        ids: vec![],
                        all: true,
                    });

                Effects::none()
                /*
                // @TODO
                let ft = api_fetch::<Env, Vec<LibItem>, _>(get_req)
                    .and_then(enclose!((key) move |items| {
                        let bucket = LibBucket { key: key.to_owned(), items: items.clone() };
                        Env::set_storage(COLL_NAME, Some(&bucket))
                            .map(move |_| LibLoaded(key, items).into())
                            .map_err(Into::into)
                    }))
                    .map_err(|e| LibFatal(e).into());

                Effects::one(Box::new(ft))
                */
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
            LibraryLoadable::Ready(lib_bucket) => {
                match msg {
                    // User actions
                    /*
                    Msg::Action(Action::UserOp(action)) => {
                        let auth = match &content.auth {
                            Some(auth) => auth,
                            None => return Effects::none().unchanged(),
                        };
                        match action {
                            ActionUser::LibSync => {
                                // @TODO get rid of the repeated map_err closure
                                let ft = lib_sync::<Env>(auth)
                                    .map(|bucket| LibSyncPulled(bucket).into())
                                    .map_err(
                                        enclose!((action) move |e| CtxActionErr(action, e).into()),
                                    );
                                Effects::one(Box::new(ft)).unchanged()
                            }
                            ActionUser::LibUpdate(item) => {
                                let bucket = LibBucket::new(&content.auth, vec![item.clone()]);
                                let ft = update_and_persist::<Env>(&mut lib_bucket, bucket)
                                    .map(|_| LibPersisted.into())
                                    .map_err(
                                        enclose!((action) move |e| CtxActionErr(action, e).into()),
                                    );
                                let push_ft =
                                    lib_push::<Env>(&item).map(|_| LibPushed.into()).map_err(
                                        enclose!((action) move |e| CtxActionErr(action, e).into()),
                                    );
                                Effects::many(vec![Box::new(persist_ft), Box::new(push_ft)])
                            }
                            _ => Effects::none().unchanged(),
                        }
                    }
                    Msg::Internal(LibSyncPulled(new_bucket)) => {
                        let ft = update_and_persist::<Env>(&mut lib_bucket, new_bucket)
                            .map(|_| LibPersisted.into())
                            .map_err(move |e| LibFatal(e).into());
                        Effects::one(Box::new(ft))
                    }
                    */
                    _ => Effects::none().unchanged(),
                }
            }
            // Ignore NotLoaded state: the load_* methods are supposed to take us out of it
            _ => Effects::none().unchanged(),
        }
    }
}

/*
fn datastore_req_builder() -> DatastoreReqBuilder {
    DatastoreReqBuilder::default()
        .auth_key(self.key.to_owned())
        .collection(COLL_NAME.to_owned())
        .clone()
}

fn lib_sync<Env: Environment + 'static>(
    auth: &Auth,
) -> impl Future<Item = LibBucket, Error = CtxError> {
    let local_lib = self.items.clone();
    let builder = self.datastore_req_builder();
    let meta_req = builder.clone().with_cmd(DatastoreCmd::Meta {});

    api_fetch::<Env, Vec<LibMTime>, _>(meta_req).and_then(move |remote_mtimes| {
        let map_remote = remote_mtimes
            .into_iter()
            .map(|LibMTime(k, mtime)| (k, mtime))
            .collect::<HashMap<_, _>>();
        // IDs to pull
        let ids = map_remote
            .iter()
            .filter(|(k, v)| local_lib.get(*k).map_or(true, |item| item.mtime < **v))
            .map(|(k, _)| k.clone())
            .collect::<Vec<String>>();
        // Items to push
        let changes = local_lib
            .iter()
            .filter(|(id, item)| {
                map_remote.get(*id).map_or(true, |date| *date < item.mtime) && item.should_push()
            })
            .map(|(_, v)| v.clone())
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

        get_fut.join(put_fut).map(|(items, _)| items)
    })
}

fn lib_push<Env: Environment + 'static>(
    auth: &Auth,
    item: &LibItem,
) -> impl Future<Item = (), Error = CtxError> {
    let push_req = self.datastore_req_builder().with_cmd(DatastoreCmd::Put {
        changes: vec![item.to_owned()],
    });

    api_fetch::<Env, SuccessResponse, _>(push_req).map(|_| ())
}

fn update_and_persist<Env: Environment + 'static>(
    bucket: &mut LibBucket,
    new_bucket: &LibBucket,
) -> impl Future<Item = (), Error = CtxError> {
    Env::set_storage(COLL_NAME, Some(&bucket)).map_err(Into::into)
}
*/
