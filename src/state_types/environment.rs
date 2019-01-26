use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;

pub use http::{Method, Request};

pub type EnvFuture<T> = Box<Future<Item = T, Error = Box<Error>>>;
pub trait Environment {
    // https://serde.rs/lifetimes.html#trait-bounds
    fn fetch_serde<IN, OUT>(request: &Request<IN>) -> EnvFuture<Box<OUT>>
    where
        IN: 'static + Serialize,
        OUT: 'static + DeserializeOwned;
    fn exec(fut: Box<Future<Item = (), Error = ()>>);
    fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<Box<T>>>;
    fn set_storage<T: 'static + Serialize>(key: &str, value: &T) -> EnvFuture<()>;
}
