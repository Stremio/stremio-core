use serde_derive::*;

mod state_container;
pub use self::state_container::*;

mod catalogs;
pub use self::catalogs::*;

mod actions;
pub use self::actions::*;

mod middleware;
pub use self::middleware::*;

use futures::Future;
use std::error::Error;
pub trait Environment {
    // @TODO: finalize this trait
    // @TODO: we should use `impl Trait` once it's allowed: https://github.com/rust-lang/rust/issues/34511
    fn fetch(url: String) -> Box<dyn Future<Item=Vec<u8>, Error=Error>>;
    //fn storage_get
    //fn storage_set
}
