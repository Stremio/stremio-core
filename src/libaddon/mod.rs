use crate::types::{LibItem, LibItemPreview};
use crate::addon_transport::AddonTransport;
use crate::state_types::{EnvFuture, Handler};
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use futures::future::Shared;

type SharedIndex = Arc<Mutex<Vec<LibItemPreview>>>;
struct LibAddon {
    pub idx_loader: Shared<EnvFuture<SharedIndex>>,
}

// when getting a catalog: query index, and get all full libitems with that ID (up to 100)
// when getting detail: get ID from storage directly
// when adding to library: get ID from storage, .unwrap_or(Default::default()).unremove().save() ;
//   and add to idx
//   and push to API

// add_to_idx: will get the index; update the index; if the item ID was not in the index, persist
// the ids array

// to load the index, we will use a Shared future

/*
impl Handler for LibAddon {
    // will handle all actions here
}

impl AddonTransport for LibAddon {
    // will handle all addon stuff here
}*/
