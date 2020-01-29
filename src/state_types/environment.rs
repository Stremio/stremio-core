use crate::addon_transport::{AddonHTTPTransport, AddonInterface};
use crate::constants::API_URL;
use chrono::{DateTime, Utc};
use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;

pub use http::Request;

pub type EnvError = Box<dyn Error>;

pub type EnvFuture<T> = Box<dyn Future<Item = T, Error = EnvError>>;

pub trait Environment {
    fn fetch_serde<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: 'static + Serialize,
        OUT: 'static + DeserializeOwned;
    fn exec(fut: Box<dyn Future<Item = (), Error = ()>>);
    fn now() -> DateTime<Utc>;
    fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<T>>;
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
