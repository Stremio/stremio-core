use crate::state_types::Event::*;
use crate::state_types::Internal::*;
use crate::state_types::*;
use crate::types::LibItem;
use crate::types::api::*;
use std::collections::HashMap;
use super::Auth;

pub type LibraryIndex = HashMap<String, LibItem>;

// Implementing Auth here is a bit unconventional,
// but rust allows multiple impl blocks precisely to allow
// separation of concerns like this
impl Auth {
    pub fn lib_update(&mut self, items: Vec<LibItem>) {
        unimplemented!()
    }
    pub fn lib_sync(&self) -> EnvFuture<Vec<LibItem>> {
        unimplemented!()
    }
    fn lib_push(&self, item: &LibItem) -> EnvFuture<()> {
        unimplemented!()
    }
    fn lib_pull(&self, id: &str) -> EnvFuture<LibItem> {
        unimplemented!()
    }
}
