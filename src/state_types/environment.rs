use crate::addon_transport::{AddonHTTPTransport, AddonInterface};
use crate::types::addons::TransportUrl;
use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;

pub use http::Request;

const API_URL: &str = "https://api.strem.io";

pub type EnvError = Box<dyn Error>;
pub type EnvFuture<T> = Box<dyn Future<Item = T, Error = EnvError>>;
pub trait Environment {
    // https://serde.rs/lifetimes.html#trait-bounds
    fn fetch_serde<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: 'static + Serialize,
        OUT: 'static + DeserializeOwned;
    fn exec(fut: Box<dyn Future<Item = (), Error = ()>>);
    fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<T>>;
    fn set_storage<T: 'static + Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()>;
    fn addon_transport(url: &TransportUrl) -> Box<dyn AddonInterface>
    where
        Self: Sized + 'static,
    {
        Box::new(AddonHTTPTransport::<Self>::from_url(url))
    }
    fn api_url() -> &'static str {
        API_URL
    }
}

//pub trait Player {
//  fn dispatch(&self, msg: &PlayerMsg) -> Box<dyn Future<Item = (), Error = ()>>;
//  fn observe(&self) -> Box<dyn Stream<Item = PlayerMsg>>;
//}
