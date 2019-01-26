use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
pub use reqwest::{Request, RequestBuilder};
pub type EnvFuture<T> = Box<Future<Item = T, Error = Box<Error>>>;
pub trait Environment {
    // https://serde.rs/lifetimes.html#trait-bounds
    fn fetch_serde<I: 'static + Serialize, T: 'static + DeserializeOwned>(url: &str, body: I) -> EnvFuture<Box<T>>;
    fn exec(fut: Box<Future<Item = (), Error = ()>>);
    fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<Box<T>>>;
    fn set_storage<T: 'static + Serialize>(key: &str, value: &T) -> EnvFuture<()>;
}
