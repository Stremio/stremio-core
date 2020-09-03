use crate::addon_transport::{AddonHTTPTransport, AddonInterface};
use crate::constants::API_URL;
use chrono::{DateTime, Utc};
use core::pin::Pin;
use futures::Future;
use http::Request;
use serde::{Deserialize, Serialize};
use std::error::Error;
use url::Url;

pub type EnvError = Box<dyn Error>;

pub type EnvFuture<T> = Pin<Box<dyn Future<Output = Result<T, EnvError>> + Unpin>>;

pub trait Environment {
    fn fetch<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: 'static + Serialize,
        for<'de> OUT: 'static + Deserialize<'de>;
    fn exec(future: Pin<Box<dyn Future<Output = ()> + Unpin>>);
    fn now() -> DateTime<Utc>;
    fn get_storage<T>(key: &str) -> EnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static;
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()>;
    fn addon_transport(url: &Url) -> Box<dyn AddonInterface>
    where
        Self: Sized + 'static,
    {
        Box::new(AddonHTTPTransport::<Self>::from_url(url))
    }
    fn api_url() -> &'static Url {
        &API_URL
    }
}
