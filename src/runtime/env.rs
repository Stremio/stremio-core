use crate::addon_transport::{AddonHTTPTransport, AddonTransport, UnsupportedTransport};
use crate::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY, SCHEMA_VERSION,
    SCHEMA_VERSION_STORAGE_KEY,
};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
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
    StorageSchemaVersionUpgrade(Box<EnvError>),
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
            EnvError::StorageSchemaVersionUpgrade(source) => format!(
                "Upgrade storage schema version failed caused by: {}",
                source.message()
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
            EnvError::StorageSchemaVersionUpgrade(_) => 3,
            EnvError::Fetch(_) => 4,
            EnvError::AddonTransport(_) => 5,
            EnvError::Serde(_) => 6,
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

pub trait Env {
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
    fn addon_transport(transport_url: &Url) -> Box<dyn AddonTransport>
    where
        Self: Sized + 'static,
    {
        match transport_url.scheme() {
            "http" | "https" => Box::new(AddonHTTPTransport::<Self>::new(transport_url.to_owned())),
            _ => Box::new(UnsupportedTransport::new(transport_url.to_owned())),
        }
    }
    fn migrate_storage_schema() -> EnvFuture<()>
    where
        Self: Sized,
    {
        Self::get_storage::<usize>(SCHEMA_VERSION_STORAGE_KEY)
            .and_then(|schema_version| async move {
                let mut schema_version = schema_version.unwrap_or_default();
                if schema_version > SCHEMA_VERSION {
                    return Err(EnvError::StorageSchemaVersionDowngrade(
                        schema_version,
                        SCHEMA_VERSION,
                    ));
                };
                if schema_version == 0 {
                    migrate_storage_schema_to_v1::<Self>()
                        .map_err(|error| EnvError::StorageSchemaVersionUpgrade(Box::new(error)))
                        .await?;
                    schema_version = 1;
                };
                // TODO v2
                // if schema_version == 1 {
                //     migrate_storage_schema_to_v2::<Self>()
                //         .map_err(|error| EnvError::StorageSchemaVersionUpgrade(Box::new(error)))
                //         .await?;
                //     schema_version = 2;
                // };
                if schema_version != SCHEMA_VERSION {
                    panic!(
                        "Storage schema version must be upgraded from {} to {}",
                        schema_version, SCHEMA_VERSION
                    );
                };
                Ok(())
            })
            .boxed_local()
    }
}

fn migrate_storage_schema_to_v1<E: Env>() -> EnvFuture<()> {
    future::try_join_all(vec![
        E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&1)),
        E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
        E::set_storage::<()>(LIBRARY_RECENT_STORAGE_KEY, None),
        E::set_storage::<()>(LIBRARY_STORAGE_KEY, None),
    ])
    .map_ok(|_| ())
    .boxed_local()
}
