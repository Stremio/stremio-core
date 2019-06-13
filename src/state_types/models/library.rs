use crate::state_types::Event::*;
use crate::state_types::Internal::*;
use crate::state_types::*;
use crate::types::LibItem;
use crate::types::api::*;
use std::collections::HashMap;
use futures::future::Future;
use super::{Auth, api_fetch};

const COLL_NAME: &str = "libraryItem";

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
    pub fn lib_sync<Env: Environment + 'static>(&self) -> EnvFuture<Vec<LibItem>> {
        /*
        let api_req = APIRequest::DatastoreMeta {
            auth_key: self.key.clone(),
            collection: COLL_NAME.into(),
        };
        let url = format!("{}/api/{}", Env::api_url(), api_req.method_name());
        let req = Request::post(url)
            .body(api_req)
            .expect("builder cannot fail");

        // @TODO datastoreMeta first
        // then dispatch the datastorePut/datastoreGet simultaniously
        let ft = api_fetch::<Env, APIResult<Vec<LibMTime>>>(api_req).then(move |resp| {
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
        // @TODO better err handling
        let ft = api_fetch::<Env, Vec<LibItem>>(req)
            .map_err(|_| "oof".into());
        Box::new(ft)
    }
    fn lib_push(&self, item: &LibItem) -> EnvFuture<()> {
        unimplemented!()
    }
    fn lib_pull(&self, id: &str) -> EnvFuture<LibItem> {
        unimplemented!()
    }
}
