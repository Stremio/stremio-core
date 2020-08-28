use crate::addon_transport::{AddonHTTPTransport, AddonInterface};
use crate::constants::API_URL;
use chrono::{DateTime, Utc};
use futures::Future;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub use http::Request;

pub type EnvError = Box<dyn Error>;

pub type EnvFuture<T> = Box<dyn Future<Item = T, Error = EnvError>>;

pub trait Environment {
    fn fetch<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: 'static + Serialize,
        for<'de> OUT: 'static + Deserialize<'de>;
    fn exec(fut: Box<dyn Future<Item = (), Error = ()>>);
    fn now() -> DateTime<Utc>;
    fn get_storage<T>(key: &str) -> EnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static;
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()>;
    fn addon_transport(url: &str) -> Box<dyn AddonInterface>
    where
        Self: Sized + 'static,
    {
        Box::new(AddonHTTPTransport::<Self>::from_url(url))
    }
    fn api_url() -> &'static str {
        API_URL
    }
}
