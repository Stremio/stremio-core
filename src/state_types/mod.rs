use serde_derive::*;

mod state_container;
pub use self::state_container::*;

mod catalogs;
pub use self::catalogs::*;

mod actions;
pub use self::actions::*;

mod middleware;
pub use self::middleware::*;

use std::error::Error;
use futures::Future;
pub trait Environment {
    fn fetch(url: String) -> Box<Future<Item=Box<Vec<u8>>, Error=Box<Error>>>;
    // @TODO: get_storage, set_storage
}
