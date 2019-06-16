use crate::state_types::*;
use crate::state_types::Internal::*;
use crate::state_types::Event::*;
use crate::types::api::*;
use crate::types::LibItem;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use futures::Future;
use serde_derive::*;
use derivative::*;
use std::collections::HashMap;
use enclose::*;

const COLL_NAME: &str = "libraryItem";

type UID = Option<String>;

#[derive(Debug, Deserialize)]
struct LibMTime(String, #[serde(with = "ts_milliseconds")] DateTime<Utc>);

// @TODO
// userless lib: load_initial_api will just be renamed to load_initial and might skip loading from
//   we may get rid of the Library type
// update should be a match on LibraryLoadable first
// API design when there is no user: we won't handle UserOp, so only way to modify lib items will
// be through the methods on LibraryLoadable


#[derive(Serialize, Deserialize, Default)]
struct LibBucket {
    key: AuthKey,
    items: Vec<LibItem>,
}

#[derive(Derivative)]
#[derivative(Debug, Default, Clone)]
pub enum LibraryLoadable {
    // NotLoaded: we've never attempted loading the library index
    #[derivative(Default)]
    NotLoaded,
    // You can't have a library without being logged in
    // OR CAN YOU??
    // @TODO
    NoUser,
    // currently being loaded for this auth key, either cause a user just logged in,
    // or cause we're loading from storage
    // @TODO explain this better
    // @TODO should this be uid? it probably should cause storage bucket will use UID
    Loading(AuthKey),
    Ready(Library),
}

impl LibraryLoadable {
    pub fn load_from_storage<Env: Environment + 'static>(&mut self, content: &CtxContent) -> Effects {
        *self = match &content.auth {
            Some(a) => LibraryLoadable::Loading(a.key.to_owned()),
            None => LibraryLoadable::NoUser,
        };
        // @TODO load recent bucket
        let ft = Env::get_storage::<LibBucket>(COLL_NAME)
            .map(|s| s.unwrap_or_default())
            .map(|LibBucket { key, items }| LibLoaded(key, items).into())
            .map_err(|e| LibFatal(e.into()).into());
        Effects::one(Box::new(ft))
    }
    pub fn load_initial_api<Env: Environment + 'static>(&mut self, content: &CtxContent) -> Effects {
        *self = match &content.auth {
            Some(a) => LibraryLoadable::Loading(a.key.to_owned()),
            None => LibraryLoadable::NoUser,
        };
        
        match &content.auth {
            None => Effects::none(),
            Some(a) => {
                let key = &a.key;
                let get_req = DatastoreReqBuilder::default()
                    .auth_key(key.to_owned())
                    .collection(COLL_NAME.to_owned())
                    .with_cmd(DatastoreCmd::Get { ids: vec![], all: true });

                let ft = api_fetch::<Env, Vec<LibItem>, _>(get_req)
                    .and_then(enclose!((key) move |items| {
                        let bucket = LibBucket { key: key.to_owned(), items: items.clone() };
                        Env::set_storage(COLL_NAME, Some(&bucket))
                            .map(move |_| LibLoaded(key, items).into())
                            .map_err(Into::into)
                    }))
                    .map_err(|e| LibFatal(e).into());

                Effects::one(Box::new(ft))
            }
        }
    }
    pub fn update<Env: Environment + 'static>(&mut self, msg: &Msg) -> Effects {
        match self {
            LibraryLoadable::Loading(loading_key) => {
                match msg {
                    Msg::Internal(LibLoaded(key, items)) if key == loading_key => {
                        let mut lib = Library {
                            key: key.to_owned(),
                            items: items.iter().map(|i| (i.id.clone(), i.clone())).collect()
                        };
                        *self = LibraryLoadable::Ready(lib);
                        Effects::none()
                    }
                    _ => Effects::none().unchanged()
                }
            }
            LibraryLoadable::Ready(lib) => {
                match msg {
                    // User actions
                    Msg::Action(Action::UserOp(action)) => match action.to_owned() {
                        ActionUser::LibSync => {
                            let key = &lib.key;
                            let action = action.to_owned();
                            let ft = lib
                                .sync::<Env>()
                                .map(enclose!((key) move |items| LibSyncPulled(key, items).into()))
                                .map_err(move |e| CtxActionErr(action, e.into()).into());
                            Effects::one(Box::new(ft)).unchanged()
                        },
                        ActionUser::LibUpdate(item) => {
                            lib.items.insert(item.id.clone(), item.clone());
                            let persist_ft = lib.persist::<Env>()
                                .map(|_| LibPersisted.into())
                                .map_err(enclose!((action) move |e| CtxActionErr(action, e).into()));
                            let push_ft = lib.push::<Env>(&item)
                                .map(|_| LibPushed.into())
                                .map_err(enclose!((action) move |e| CtxActionErr(action, e).into()));
                            Effects::many(vec![
                                Box::new(persist_ft),
                                Box::new(push_ft)
                            ])
                        }
                        _ => Effects::none().unchanged()
                    }
                    Msg::Internal(LibSyncPulled(key, items)) if key == &lib.key => {
                        for item in items.iter() {
                            lib.items.insert(item.id.clone(), item.clone());
                        }
                        let ft = lib.persist::<Env>()
                            .map(|_| LibPersisted.into())
                            .map_err(move |e| LibFatal(e).into());
                        Effects::one(Box::new(ft))
                    }
                    _ => Effects::none().unchanged(),
                }

            }
            // Ignore NotLoaded state: the load_* methods are supposed to take us out of it
            _ => Effects::none().unchanged()
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Library {
    pub key: AuthKey,
    pub items: HashMap<String, LibItem>,
}

impl Library {
    fn datastore_req_builder(&self) -> DatastoreReqBuilder {
        DatastoreReqBuilder::default()
            .auth_key(self.key.to_owned())
            .collection(COLL_NAME.to_owned())
            .clone()
    }
    pub fn sync<Env: Environment + 'static>(
        &self,
    ) -> impl Future<Item = Vec<LibItem>, Error = CtxError> {
        let local_lib = self.items.clone();
        let builder = self.datastore_req_builder();
        let meta_req = builder.clone().with_cmd(DatastoreCmd::Meta {});
        let ft = api_fetch::<Env, Vec<LibMTime>, _>(meta_req).and_then(move |remote_mtimes| {
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
                    map_remote.get(*id).map_or(true, |date| *date < item.mtime)
                        && item.should_push()
                })
                .map(|(_, v)| v.clone())
                .collect::<Vec<LibItem>>();
            let get_req = builder
                .clone()
                .with_cmd(DatastoreCmd::Get { ids, all: false });
            let put_req = builder.clone().with_cmd(DatastoreCmd::Put { changes });
            api_fetch::<Env, Vec<LibItem>, _>(get_req)
                .join(api_fetch::<Env, SuccessResponse, _>(put_req))
                .map(|(items, _)| items)
        });
        Box::new(ft)
    }
    pub fn push<Env: Environment + 'static>(
        &self,
        item: &LibItem,
    ) -> impl Future<Item = (), Error = CtxError> {
        let push_req = self
            .datastore_req_builder()
            .with_cmd(DatastoreCmd::Put {
                changes: vec![item.to_owned()],
            });

        api_fetch::<Env, SuccessResponse, _>(push_req).map(|_| ())
    }
    pub fn persist<Env: Environment + 'static>(&self) -> impl Future<Item = (), Error = CtxError> {
        let bucket = LibBucket {
            key: self.key.clone(),
            items: self.items.iter().map(|(_, v)| v).cloned().collect()
        };
        Env::set_storage(COLL_NAME, Some(&bucket))
            .map_err(Into::into)
    }
}
