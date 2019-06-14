use crate::state_types::Event::*;
use crate::state_types::Internal::*;
use crate::state_types::*;
use crate::types::LibItem;
use crate::types::api::*;
use std::collections::HashMap;
use futures::{Future, future};
use super::{Auth, api_fetch};

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

// Implementing Auth here is a bit unconventional,
// but rust allows multiple impl blocks precisely to allow
// separation of concerns like this
impl Auth {
    pub fn lib_update(&mut self, items: &[LibItem]) {
        for item in items.iter() {
            self.lib.insert(item.id.to_owned(), item.to_owned());
        }
    }
    // @TODO rather than EnvFuture, use a Future that returns CtxError
    pub fn lib_sync<Env: Environment + 'static>(&self) -> impl Future<Item = Vec<LibItem>, Error = CtxError> {
        /*
        let api_req = APIRequest::DatastoreMeta {
            auth_key: self.key.clone(),
            collection: COLL_NAME.into(),
        };
        // @TODO datastoreMeta first
        // then dispatch the datastorePut/datastoreGet simultaniously
        let ft = api_fetch::<Env, Vec<LibMTime>>(api_req).then(move |resp| {
            let map_remote = result
                .into_iter()
                .map(|LibMTime(k, mtime)| (k, mtime))
                .collect::<HashMap<String, DateTime<Utc>>>();
            let to_pull_ids = map_remote
                .iter()
                .filter(|(k, v)| idx.get(*k).map_or(true, |item| item.mtime < **v))
                .map(|(k, _)| k.to_owned())
                .collect::<Vec<String>>();
            let to_push = idx
                .iter()
                .filter(|(id, item)| {
                    map_remote.get(*id).map_or(true, |date| *date < item.mtime)
                })
                .map(|(_, v)| v)
                .collect::<Vec<&LibItem>>();
        });
        */
        let req = APIRequest::DatastoreGet {
            auth_key: self.key.clone(),
            collection: COLL_NAME.into(),
            all: true,
            ids: vec![],
        };
        let ft = api_fetch::<Env, Vec<LibItem>>(req);
        Box::new(ft)
    }
    fn lib_push(&self, item: &LibItem) -> impl Future<Item = (), Error = CtxError> {
        unimplemented!();
        future::ok(())
    }
    fn lib_pull(&self, id: &str) -> impl Future<Item = Option<LibItem>, Error = CtxError> {
        unimplemented!();
        future::ok(None)
    }
}
