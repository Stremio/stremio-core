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

const COLL_NAME: &str = "libraryItem";

#[derive(Debug, Deserialize)]
struct LibMTime(String, #[serde(with = "ts_milliseconds")] DateTime<Utc>);

// @TODO
// persist effect - triggered on LibUpdate/LibSyncPulled; will end in LibPersisted
// load_initial_api - will do datastoreGet, persist in full, and end in LibLoaded
// load_from_storage - will load storage buckets, and end in LibLoaded
// userless lib: load_initial_api will just be renamed to load_initial and might skip loading from
// API if there is no user
// update should be a match on LibraryLoadable first


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
                let key = a.key.to_owned();
                let get_req = DatastoreReqBuilder::default()
                    .auth_key(key.to_owned())
                    .collection(COLL_NAME.to_owned())
                    .with_cmd(DatastoreCmd::Get { ids: vec![], all: true });

                let ft = api_fetch::<Env, Vec<LibItem>, _>(get_req)
                    .map(move |items| LibLoaded(key, items).into())
                    .map_err(|e| CtxFatal(e.into()).into());

                Effects::one(Box::new(ft))
            }
        }
    }
    pub fn update<Env: Environment + 'static>(&mut self, msg: &Msg) -> Effects {
        // is not loaded, and content is_loaded
        //*self = LibraryLoadable::NotLoaded;
        // @TODO reorganize this to match on libraryindex state first
        // and then apply everything else?
        match &msg {
            Msg::Action(Action::UserOp(action)) => match action.to_owned() {
                ActionUser::LibSync => match self {
                    LibraryLoadable::Ready(lib) => {
                        let key = lib.key.to_owned();
                        let action = action.to_owned();
                        let ft = lib
                            .sync::<Env>()
                            .map(move |items| LibSyncPulled(key, items).into())
                            .map_err(move |e| CtxActionErr(action, e.into()).into());
                        Effects::one(Box::new(ft)).unchanged()
                    }
                    _ => Effects::none().unchanged(),
                },
                ActionUser::LibUpdate(item) => match self {
                    LibraryLoadable::Ready(lib) => {
                        // @TODO
                        lib.items.insert(item.id.clone(), item.clone());
                        Effects::none()
                    }
                    _ => Effects::none().unchanged(),
                }
                _ => Effects::none().unchanged(),
            }
            Msg::Internal(LibLoaded(key, items)) => match self {
                LibraryLoadable::Loading(loading_key) if key == loading_key => {
                    let mut lib = Library {
                        key: key.to_owned(),
                        items: Default::default()
                    };
                    lib.update(items);
                    *self = LibraryLoadable::Ready(lib);
                    Effects::none()
                },
                _ => Effects::none().unchanged(),
            }
            Msg::Internal(LibSyncPulled(key, items)) => match self {
                LibraryLoadable::Ready(lib) if key == &lib.key => {
                    lib.update(items);
                    // @TODO persist
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            }
            _ => Effects::none().unchanged(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Library {
    pub key: AuthKey,
    // @TODO the state should be NotLoaded, Loading, Ready; so that when we dispatch LibSync we
    // can ensure we've waited for storage load first
    // perhaps wrap it on a Ctx level, and have an effect for loading from storage here; this
    // effect can either fail massively (CtxFatal) or succeed
    // when the user is logged out, we'll reset it to NotLoaded
    pub items: HashMap<String, LibItem>,
}

impl Library {
    pub fn update(&mut self, items: &[LibItem]) {
        for item in items.iter() {
            self.items.insert(item.id.clone(), item.clone());
        }
    }
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
        // @TODO review existing sync logic and see how this differs
        // @TODO respect .should_push()
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
}
