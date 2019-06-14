use crate::state_types::*;
use crate::types::LibItem;
use crate::types::api::*;
use std::collections::HashMap;
use futures::{Future, future};
use super::{Auth, api_fetch};
use serde_derive::*;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};

const COLL_NAME: &str = "libraryItem";

/*
pub struct Library {
    // @TODO the state should be NotLoaded, Loading, Ready; so that when we dispatch LibSync we
    // can ensure we've waited for storage load first
    // perhaps wrap it on a Ctx level, and have an effect for loading from storage here; this
    // effect can either fail massively (CtxFatal) or succeed
    // when the user is logged out, we'll reset it to NotLoaded
    pub items: HashMap<String, LibItem>,
    pub last_videos: Vec<MetaDetail>,
}
*/
pub type LibraryIndex = HashMap<String, LibItem>;

#[derive(Debug, Deserialize)]
struct LibMTime(String, #[serde(with = "ts_milliseconds")] DateTime<Utc>);

// Implementing Auth here is a bit unconventional,
// but rust allows multiple impl blocks precisely to allow
// separation of concerns like this
impl Auth {
    pub fn lib_update(&mut self, items: &[LibItem]) {
        for item in items.iter() {
            self.lib.insert(item.id.clone(), item.clone());
        }
    }
    fn lib_datastore_req_builder(&self) -> DatastoreReqBuilder {
        DatastoreReqBuilder::default()
            .auth_key(self.key.to_owned())
            .collection(COLL_NAME.to_owned())
            .clone()
    }
    // @TODO rather than EnvFuture, use a Future that returns CtxError
    pub fn lib_sync<Env: Environment + 'static>(&self) -> impl Future<Item = Vec<LibItem>, Error = CtxError> {
        let local_lib = self.lib.clone();
        let builder = self.lib_datastore_req_builder();
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
                })
                .map(|(_, v)| v.clone())
                .collect::<Vec<LibItem>>();
            let get_req = builder.clone().with_cmd(DatastoreCmd::Get { ids, all: false });
            let put_req = builder.clone().with_cmd(DatastoreCmd::Put { changes });
            api_fetch::<Env, Vec<LibItem>, _>(get_req)
                .join(api_fetch::<Env, SuccessResponse, _>(put_req))
                .map(|(items, _)| items)
        });
        Box::new(ft)
    }
    fn lib_push<Env: Environment + 'static>(&self, item: &LibItem) -> impl Future<Item = (), Error = CtxError> {
        let push_req = self.lib_datastore_req_builder()
            .with_cmd(DatastoreCmd::Put { changes: vec![item.to_owned()] });

        api_fetch::<Env, SuccessResponse, _>(push_req)
            .map(|_| ())
    }
    fn lib_pull<Env: Environment + 'static>(&self, id: &str) -> impl Future<Item = Option<LibItem>, Error = CtxError> {
        let pull_req = self.lib_datastore_req_builder()
            .with_cmd(DatastoreCmd::Get { all: false, ids: vec![id.to_owned()] });

        api_fetch::<Env, Vec<LibItem>, _>(pull_req)
            .map(|mut items| items.pop())
    }
}
