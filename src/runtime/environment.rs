use crate::addon_transport::{AddonHTTPTransport, AddonTransport};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use futures::Future;
use http::Request;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use url::Url;

#[derive(Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum EnvError {
    StorageUnavailable,
    StorageSchemaVersionDowngrade(usize, usize),
    Fetch(String),
    AddonTransport(String),
    Serde(String),
}

impl EnvError {
    pub fn message(&self) -> String {
        match &self {
            EnvError::StorageUnavailable => "Storage is not available.".to_owned(),
            EnvError::StorageSchemaVersionDowngrade(from, to) => format!(
                "Downgrade storage schema version from {} to {} is not allowed.",
                from, to
            ),
            EnvError::Fetch(message) => format!("Failed to fetch: {}", message),
            EnvError::AddonTransport(message) => format!("Addon protocol violation: {}", message),
            EnvError::Serde(message) => format!("Serialization error: {}", message),
        }
    }
    pub fn code(&self) -> u64 {
        match &self {
            EnvError::StorageUnavailable => 1,
            EnvError::StorageSchemaVersionDowngrade(_, _) => 2,
            EnvError::Fetch(_) => 3,
            EnvError::AddonTransport(_) => 4,
            EnvError::Serde(_) => 5,
        }
    }
}

impl Serialize for EnvError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EnvError", 2)?;
        state.serialize_field("code", &self.code())?;
        state.serialize_field("message", &self.message())?;
        state.end()
    }
}

pub type EnvFuture<T> = LocalBoxFuture<'static, Result<T, EnvError>>;

pub trait Environment {
    fn fetch<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: Serialize + 'static,
        for<'de> OUT: Deserialize<'de> + 'static;
    fn get_storage<T>(key: &str) -> EnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static;
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()>;
    fn exec<F>(future: F)
    where
        F: Future<Output = ()> + 'static;
    fn now() -> DateTime<Utc>;
    fn addon_transport(transport_url: &Url) -> Box<dyn AddonTransport>
    where
        Self: Sized + 'static,
    {
        Box::new(AddonHTTPTransport::<Self>::new(transport_url))
    }
}
