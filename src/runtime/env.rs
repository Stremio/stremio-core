use crate::addon_transport::{AddonHTTPTransport, AddonTransport, UnsupportedTransport};
use crate::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY, SCHEMA_VERSION,
    SCHEMA_VERSION_STORAGE_KEY,
};
use crate::models::ctx::Ctx;
use crate::models::streaming_server::StreamingServer;
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use futures::{future, Future, FutureExt, TryFutureExt};
use http::Request;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use url::Url;

#[derive(Clone, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum EnvError {
    Fetch(String),
    AddonTransport(String),
    Serde(String),
    StorageUnavailable,
    StorageSchemaVersionDowngrade(u32, u32),
    StorageSchemaVersionUpgrade(Box<EnvError>),
}

impl EnvError {
    pub fn message(&self) -> String {
        match &self {
            EnvError::Fetch(message) => format!("Failed to fetch: {}", message),
            EnvError::AddonTransport(message) => format!("Addon protocol violation: {}", message),
            EnvError::Serde(message) => format!("Serialization error: {}", message),
            EnvError::StorageUnavailable => "Storage is not available".to_owned(),
            EnvError::StorageSchemaVersionDowngrade(from, to) => format!(
                "Downgrade storage schema version from {} to {} is not allowed",
                from, to
            ),
            EnvError::StorageSchemaVersionUpgrade(source) => format!(
                "Upgrade storage schema version failed caused by: {}",
                source.message()
            ),
        }
    }
    pub fn code(&self) -> u64 {
        match &self {
            EnvError::Fetch(_) => 1,
            EnvError::AddonTransport(_) => 2,
            EnvError::Serde(_) => 3,
            EnvError::StorageUnavailable => 4,
            EnvError::StorageSchemaVersionDowngrade(_, _) => 5,
            EnvError::StorageSchemaVersionUpgrade(_) => 6,
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

pub type EnvFuture<T> = LocalBoxFuture<'static, T>;

pub type TryEnvFuture<T> = EnvFuture<Result<T, EnvError>>;

pub trait Env {
    fn fetch<IN, OUT>(request: Request<IN>) -> TryEnvFuture<OUT>
    where
        IN: Serialize,
        for<'de> OUT: Deserialize<'de> + 'static;
    fn get_storage<T>(key: &str) -> TryEnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static;
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> TryEnvFuture<()>;
    fn exec<F>(future: F)
    where
        F: Future<Output = ()> + 'static;
    fn now() -> DateTime<Utc>;
    fn flush_analytics() -> EnvFuture<()>;
    fn analytics_context(ctx: &Ctx, streaming_server: &StreamingServer) -> serde_json::Value;
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
    fn migrate_storage_schema() -> TryEnvFuture<()>
    where
        Self: Sized,
    {
        Self::get_storage::<u32>(SCHEMA_VERSION_STORAGE_KEY)
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
                if schema_version == 1 {
                    migrate_storage_schema_to_v2::<Self>()
                        .map_err(|error| EnvError::StorageSchemaVersionUpgrade(Box::new(error)))
                        .await?;
                    schema_version = 2;
                };
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

fn migrate_storage_schema_to_v1<E: Env>() -> TryEnvFuture<()> {
    future::try_join_all(vec![
        E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&1)),
        E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
        E::set_storage::<()>(LIBRARY_RECENT_STORAGE_KEY, None),
        E::set_storage::<()>(LIBRARY_STORAGE_KEY, None),
    ])
    .map_ok(|_| ())
    .boxed_local()
}

fn migrate_storage_schema_to_v2<E: Env>() -> TryEnvFuture<()> {
    E::get_storage::<serde_json::Value>(PROFILE_STORAGE_KEY)
        .and_then(|mut profile| {
            match profile
                .as_mut()
                .and_then(|profile| profile.as_object_mut())
                .and_then(|profile| profile.get_mut("settings"))
                .and_then(|settings| settings.as_object_mut())
                .map(|settings| {
                    (
                        settings.remove("interface_language"),
                        settings.remove("streaming_server_url"),
                        settings.remove("binge_watching"),
                        settings.remove("play_in_background"),
                        settings.remove("play_in_external_player"),
                        settings.remove("hardware_decoding"),
                        settings.remove("subtitles_language"),
                        settings.remove("subtitles_size"),
                        settings.remove("subtitles_font"),
                        settings.remove("subtitles_bold"),
                        settings.remove("subtitles_offset"),
                        settings.remove("subtitles_text_color"),
                        settings.remove("subtitles_background_color"),
                        settings.remove("subtitles_outline_color"),
                        settings.remove("streaming_server_warning_dismissed"),
                        settings,
                    )
                }) {
                Some((
                    Some(interface_language),
                    Some(streaming_server_url),
                    Some(binge_watching),
                    Some(play_in_background),
                    Some(play_in_external_player),
                    Some(hardware_decoding),
                    Some(subtitles_language),
                    Some(subtitles_size),
                    Some(subtitles_font),
                    Some(subtitles_bold),
                    Some(subtitles_offset),
                    Some(subtitles_text_color),
                    Some(subtitles_background_color),
                    Some(subtitles_outline_color),
                    Some(streaming_server_warning_dismissed),
                    settings,
                )) => {
                    settings.insert("interfaceLanguage".to_owned(), interface_language);
                    settings.insert("streamingServerUrl".to_owned(), streaming_server_url);
                    settings.insert("bingeWatching".to_owned(), binge_watching);
                    settings.insert("playInBackground".to_owned(), play_in_background);
                    settings.insert("playInExternalPlayer".to_owned(), play_in_external_player);
                    settings.insert("hardwareDecoding".to_owned(), hardware_decoding);
                    settings.insert("subtitlesLanguage".to_owned(), subtitles_language);
                    settings.insert("subtitlesSize".to_owned(), subtitles_size);
                    settings.insert("subtitlesFont".to_owned(), subtitles_font);
                    settings.insert("subtitlesBold".to_owned(), subtitles_bold);
                    settings.insert("subtitlesOffset".to_owned(), subtitles_offset);
                    settings.insert("subtitlesTextColor".to_owned(), subtitles_text_color);
                    settings.insert(
                        "subtitlesBackgroundColor".to_owned(),
                        subtitles_background_color,
                    );
                    settings.insert("subtitlesOutlineColor".to_owned(), subtitles_outline_color);
                    settings.insert("streamingServerWarningDismissed".to_owned(), streaming_server_warning_dismissed);
                    E::set_storage(PROFILE_STORAGE_KEY, Some(&profile))
                }
                _ => E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
            }
        })
        .and_then(|_| E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&2)))
        .boxed_local()
}
