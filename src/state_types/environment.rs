use std::error::Error;
use futures::Future;
use serde::de::DeserializeOwned;
pub trait Environment {
    // https://serde.rs/lifetimes.html#trait-bounds
    fn fetch_serde<T: 'static>(url: String) -> Box<Future<Item=Box<T>, Error=Box<Error>>> where T: DeserializeOwned;
    fn exec(fut: Box<Future<Item=(), Error=()>>);
    // @TODO: get_storage
    // @TODO: set_storage
}
