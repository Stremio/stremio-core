//use crate::addon_transport::AddonTransport;
use crate::addon_transport::AddonInterface;
use crate::state_types::{EnvFuture, Environment};
use crate::types::addons::{Manifest, ResourceRef, ResourceResponse};
use crate::types::{LibItemPreview};
use futures::future::Shared;
use futures::{future, Future};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

type SharedIndex = Arc<Mutex<Vec<LibItemPreview>>>;

pub struct LibAddon<T: Environment + 'static> {
    pub idx_loader: Shared<EnvFuture<SharedIndex>>,
    env: PhantomData<T>,
}

impl<T: Environment + 'static> LibAddon<T> {
    pub fn new() -> Self {
        let idx = Arc::new(Mutex::new(Vec::new()));
        let fut = Box::new(future::ok(idx));
        LibAddon {
            idx_loader: Future::shared(fut),
            env: PhantomData,
        }
    }
}
impl<T: Environment + 'static> Default for LibAddon<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Environment + 'static> Clone for LibAddon<T> {
    fn clone(&self) -> Self {
        LibAddon {
            idx_loader: self.idx_loader.clone(),
            env: PhantomData,
        }
    }
}

// when getting a catalog: query index, and get all full libitems with that ID (up to 100)
// when getting detail: get ID from storage directly
// when adding to library: get ID from storage, .unwrap_or(Default::default()).unremove().save() ;
//   and add to idx
//   and push to API

// add_to_idx: will get the index; update the index; if the item ID was not in the index, persist
// the ids array

// to load the index, we will use a Shared future

// @TODO try this: create a LibAddon, write to it's index, clone it, check the index on the second
// one

// @TODO sync pipeline
// @TODO videos pipeline

/*
impl Handler for LibAddon {
    // will handle all actions here
}
*/

impl<T: Environment + 'static> AddonInterface for LibAddon<T> {
    fn get(&self, _: &ResourceRef) -> EnvFuture<ResourceResponse> {
        unimplemented!()
    }
    fn manifest(&self) -> EnvFuture<Manifest> {
        Box::new(
            self.idx_loader
                .clone()
                .and_then(|_idx| {
                    future::ok(Manifest {
                        id: "org.stremio.libitem".into(),
                        name: "Library".into(),
                        version: semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
                        // @TODO dynamic
                        resources: vec![],
                        types: vec!["movie".into()],
                        catalogs: vec![],
                        contact_email: None,
                        background: None,
                        logo: None,
                        id_prefixes: None,
                        description: None,
                    })
                    // @TODO fix by making the idx_loader result in a Cloneable error
                    // this is not trivial to fix, as the underlying error is also a Box<dyn Error>
                })
                .map_err(|_| "index loading error".into()),
        )
    }
}
