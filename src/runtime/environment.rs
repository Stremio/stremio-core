use crate::addon_transport::{AddonHTTPTransport, AddonTransport, UnsupportedTransport};
use crate::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY, SCHEMA_VERSION,
    SCHEMA_VERSION_STORAGE_KEY,
};
use chrono::{DateTime, Utc};
use futures::future::{Either, LocalBoxFuture};
use futures::{future, Future, FutureExt, TryFutureExt};
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

impl From<serde_json::Error> for EnvError {
    fn from(error: serde_json::Error) -> Self {
        EnvError::Serde(error.to_string())
    }
}

pub type EnvFuture<T> = LocalBoxFuture<'static, Result<T, EnvError>>;

pub trait Environment {
    fn fetch<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: Serialize,
        for<'de> OUT: Deserialize<'de> + 'static;
    fn get_storage<T>(key: &str) -> EnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static;
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()>;
    fn exec<F>(future: F)
    where
        F: Future<Output = ()> + 'static;
    fn now() -> DateTime<Utc>;
    #[cfg(debug_assertions)]
    fn log(message: String);
    fn migrate_storage_schema() -> EnvFuture<()>
    where
        Self: Sized + 'static,
    {
        Self::get_storage::<usize>(SCHEMA_VERSION_STORAGE_KEY)
            .and_then(|schema_version| {
                match schema_version {
                    Some(schema_version) if schema_version > SCHEMA_VERSION => {
                        Either::Left(future::err(EnvError::StorageSchemaVersionDowngrade(
                            schema_version,
                            SCHEMA_VERSION,
                        )))
                    }
                    None => Either::Right(migrate_storage_schema_v1::<Self>()),
                    // TODO Some(1) => Either::Right(migrate_storage_schema_v2::<Self>()),
                    _ => Either::Left(future::ok(())),
                }
            })
            .boxed_local()
    }
    fn addon_transport(transport_url: &Url) -> Box<dyn AddonTransport>
    where
        Self: Sized + 'static,
    {
        match transport_url.scheme() {
            "http" | "https" => Box::new(AddonHTTPTransport::<Self>::new(transport_url.to_owned())),
            _ => Box::new(UnsupportedTransport::new(transport_url.to_owned())),
        }
    }
}

fn migrate_storage_schema_v1<Env: Environment + 'static>() -> EnvFuture<()> {
    future::try_join_all(vec![
        Env::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&1)),
        Env::set_storage::<()>(PROFILE_STORAGE_KEY, None),
        Env::set_storage::<()>(LIBRARY_RECENT_STORAGE_KEY, None),
        Env::set_storage::<()>(LIBRARY_STORAGE_KEY, None),
    ])
    .map_ok(|_| ())
    .boxed_local()
}
