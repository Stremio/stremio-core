use crate::addon_transport::AddonInterface;
use crate::state_types::*;
use crate::types::addons::{Manifest, ResourceRef, ResourceResponse};
use crate::types::api::*;
use crate::types::LibItem;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use futures::future::{Shared, SharedItem};
use futures::{future, Future};
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Debug, Deserialize)]
struct LibMTime(String, #[serde(with = "ts_milliseconds")] DateTime<Utc>);

#[derive(Debug, Clone)]
pub struct LibSyncStats {
    pulled: u64,
    pushed: u64,
}

// SharedIndex is a global, thread-safe index of library items
#[derive(Clone)]
pub struct SharedIndex(Arc<RwLock<HashMap<String, LibItem>>>);

impl SharedIndex {
    fn from_map(map: HashMap<String, LibItem>) -> Self {
        SharedIndex(Arc::new(RwLock::new(map)))
    }
    // @TODO: get_resources
    // @TODO: get_catalogs
    fn get_types(&self) -> Vec<String> {
        let idx = self.0.read().expect("unable to read idx");
        let types: HashSet<_> = idx.iter().map(|(_, x)| x.type_name.to_owned()).collect();
        types.into_iter().collect()
    }
}

// The same similar pattern is used in ContextM
type MiddlewareFuture<T> = Box<dyn Future<Item = T, Error = MiddlewareError>>;

// LibAddon: the struct that represents a LibAddon for a single session (auth_key)
// can be safely cloned in order to attach to middlewares and etc.
// Implements Handle (to respond to actions) and AddonInterface
// @TODO videos pipeline
pub struct LibAddon<T: Environment + 'static> {
    pub idx_loader: Shared<EnvFuture<SharedIndex>>,
    auth_key: AuthKey,
    env: PhantomData<T>,
}

impl<Env: Environment + 'static> LibAddon<Env> {
    pub fn with_authkey(auth_key: AuthKey) -> Self {
        let key = format!("library:{}", auth_key.chars().take(6).collect::<String>());
        // flatten cause it's an Option<Vec<...>>
        let idx_loader = Env::get_storage::<Vec<String>>(&key)
            .and_then(|keys| {
                join_all(
                    keys.into_iter()
                        .flatten()
                        .map(|k| Env::get_storage::<LibItem>(&k)),
                )
            })
            .map(|items| {
                SharedIndex::from_map(
                    items
                        .into_iter()
                        .flatten()
                        .map(|i| (i.id.clone(), i))
                        .collect(),
                )
            });
        LibAddon {
            idx_loader: Future::shared(Box::new(idx_loader)),
            auth_key,
            env: PhantomData,
        }
    }
    fn with_idx(&self) -> MiddlewareFuture<SharedItem<SharedIndex>> {
        // @TODO consider a more detailed error
        // for this, we have to convert a Box<dyn Error> into a Shared error
        Box::new(self.idx_loader.clone().map_err(|_| MiddlewareError::LibIdx))
    }
    // @TODO: persist the new index, but in self.sync_with_api
    pub fn sync_with_api(&self) -> MiddlewareFuture<LibSyncStats> {
        // @TODO not hardcoded
        let base_url = "https://api.strem.io";
        let api_req = APIRequest::DatastoreMeta {
            auth_key: self.auth_key.clone(),
            collection: "libraryItem".into(),
        };
        let url = format!("{}/api/{}", &base_url, api_req.method_name());
        let req = Request::post(url)
            .body(api_req)
            .expect("builder cannot fail");

        // @TODO datastoreMeta first
        // then dispatch the datastorePut/datastoreGet simultaniously
        let sync_fut = self.with_idx().and_then(|idx| {
            Env::fetch_serde::<_, APIResult<Vec<LibMTime>>>(req).then(move |resp| {
                if let Ok(APIResult::Ok { result }) = resp {
                    // @TODO use some method on .idx
                    let idx = idx.0.read().unwrap();
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

                    return future::ok(LibSyncStats {
                        pulled: to_pull_ids.len() as u64,
                        pushed: to_push.len() as u64,
                    });
                }
                // @TODO proper error
                future::err(MiddlewareError::LibIdx)
            })
        });

        Box::new(sync_fut)
    }
    // @TODO push_to_api
}

impl<T: Environment + 'static> Clone for LibAddon<T> {
    fn clone(&self) -> Self {
        LibAddon {
            idx_loader: self.idx_loader.clone(),
            auth_key: self.auth_key.clone(),
            env: PhantomData,
        }
    }
}

impl<Env: Environment + 'static> Handler for LibAddon<Env> {
    fn handle(&self, msg: &Msg, emit: Rc<DispatcherFn>) {
        match msg {
            Msg::Action(Action::LibSync) => {
                // @TODO proper err handling
                let sync_fut = self
                    .sync_with_api()
                    .and_then(move |LibSyncStats { pushed, pulled }| {
                        emit(&Msg::Event(Event::LibSynced { pushed, pulled }));
                        future::ok(())
                    })
                    .or_else(|_e| {
                        // @TODO err handling
                        //emit(&Msg::)
                        future::err(())
                    });
                Env::exec(Box::new(sync_fut))
            }
            Msg::Action(Action::LibUpdate(_item)) => {
                // @TODO push_to_api
                // @TODO idx.update_item(); if it's new, persist it
                unimplemented!()
            }
            _ => (),
        }
    }
}

