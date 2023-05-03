use crate::addon_transport::{AddonHTTPTransport, AddonTransport, UnsupportedTransport};
use crate::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY, SCHEMA_VERSION,
    SCHEMA_VERSION_STORAGE_KEY,
};
use crate::models::ctx::Ctx;
use crate::models::streaming_server::StreamingServer;
use chrono::{DateTime, Utc};
use futures::{future, Future, TryFutureExt};
use http::Request;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use url::Url;

pub use conditional_types::{ConditionalSend, EnvFuture, EnvFutureExt};

#[derive(Clone, PartialEq, Debug)]
pub enum EnvError {
    Fetch(String),
    AddonTransport(String),
    Serde(String),
    StorageUnavailable,
    StorageSchemaVersionDowngrade(u32, u32),
    StorageSchemaVersionUpgrade(Box<EnvError>),
    StorageReadError(String),
    StorageWriteError(String),
    Other(String),
}

impl EnvError {
    pub fn message(&self) -> String {
        match &self {
            EnvError::Fetch(message) => format!("Failed to fetch: {message}"),
            EnvError::AddonTransport(message) => format!("Addon protocol violation: {message}"),
            EnvError::Serde(message) => format!("Serialization error: {message}"),
            EnvError::StorageUnavailable => "Storage is not available".to_owned(),
            EnvError::StorageSchemaVersionDowngrade(from, to) => {
                format!("Downgrade storage schema version from {from} to {to} is not allowed",)
            }
            EnvError::StorageSchemaVersionUpgrade(source) => format!(
                "Upgrade storage schema version failed caused by: {}",
                source.message()
            ),
            EnvError::StorageReadError(message) => format!("Storage read error: {message}"),
            EnvError::StorageWriteError(message) => format!("Storage write error: {message}"),
            EnvError::Other(message) => format!("Other error: {message}"),
        }
    }
    pub fn code(&self) -> u32 {
        match &self {
            EnvError::Fetch(_) => 1,
            EnvError::AddonTransport(_) => 2,
            EnvError::Serde(_) => 3,
            EnvError::StorageUnavailable => 4,
            EnvError::StorageSchemaVersionDowngrade(_, _) => 5,
            EnvError::StorageSchemaVersionUpgrade(_) => 6,
            EnvError::StorageReadError(_) => 7,
            EnvError::StorageWriteError(_) => 8,
            EnvError::Other(_) => 1001,
        }
    }
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
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

#[cfg(not(feature = "env-future-send"))]
/// Only for wasm or when `env-future-send` is not enabled
mod conditional_types {
    use futures::{future::LocalBoxFuture, Future, FutureExt};

    pub type EnvFuture<'a, T> = LocalBoxFuture<'a, T>;

    pub trait ConditionalSend {}

    impl<T> ConditionalSend for T {}

    pub trait EnvFutureExt: Future {
        fn boxed_env<'a>(self) -> EnvFuture<'a, Self::Output>
        where
            Self: Sized + 'a,
        {
            self.boxed_local()
        }
    }
}

#[cfg(feature = "env-future-send")]
/// Enabled with the feature `env-future-send` but it requires a non-wasm target!
/// It will cause a compile-time error!
mod conditional_types {
    use futures::{future::BoxFuture, Future, FutureExt};

    pub type EnvFuture<'a, T> = BoxFuture<'a, T>;

    pub trait ConditionalSend: Send {}

    impl<T> ConditionalSend for T where T: Send {}

    pub trait EnvFutureExt: Future {
        fn boxed_env<'a>(self) -> EnvFuture<'a, Self::Output>
        where
            Self: Sized + Send + 'a,
        {
            self.boxed()
        }
    }
}

impl<T: ?Sized> EnvFutureExt for T where T: Future {}

pub type TryEnvFuture<T> = EnvFuture<'static, Result<T, EnvError>>;

pub trait Env {
    fn fetch<
        IN: Serialize + ConditionalSend + 'static,
        OUT: for<'de> Deserialize<'de> + ConditionalSend + 'static,
    >(
        request: Request<IN>,
    ) -> TryEnvFuture<OUT>;

    fn get_storage<T: for<'de> Deserialize<'de> + ConditionalSend + 'static>(
        key: &str,
    ) -> TryEnvFuture<Option<T>>;
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> TryEnvFuture<()>;
    fn exec_concurrent<F: Future<Output = ()> + ConditionalSend + 'static>(future: F);
    fn exec_sequential<F: Future<Output = ()> + ConditionalSend + 'static>(future: F);
    fn now() -> DateTime<Utc>;
    fn flush_analytics() -> EnvFuture<'static, ()>;
    fn analytics_context(
        ctx: &Ctx,
        streaming_server: &StreamingServer,
        path: &str,
    ) -> serde_json::Value;
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
                if schema_version == 2 {
                    migrate_storage_schema_to_v3::<Self>()
                        .map_err(|error| EnvError::StorageSchemaVersionUpgrade(Box::new(error)))
                        .await?;
                    schema_version = 3;
                };
                if schema_version == 3 {
                    migrate_storage_schema_to_v4::<Self>()
                        .map_err(|error| EnvError::StorageSchemaVersionUpgrade(Box::new(error)))
                        .await?;
                    schema_version = 4;
                };
                if schema_version == 4 {
                    migrate_storage_schema_to_v5::<Self>()
                        .map_err(|error| EnvError::StorageSchemaVersionUpgrade(Box::new(error)))
                        .await?;
                    schema_version = 5;
                };
                if schema_version == 5 {
                    migrate_storage_schema_to_v6::<Self>()
                        .map_err(|error| EnvError::StorageSchemaVersionUpgrade(Box::new(error)))
                        .await?;
                    schema_version = 6;
                };
                if schema_version == 6 {
                    migrate_storage_schema_to_v7::<Self>()
                        .map_err(|error| EnvError::StorageSchemaVersionUpgrade(Box::new(error)))
                        .await?;
                    schema_version = 7;
                };
                if schema_version != SCHEMA_VERSION {
                    panic!(
                        "Storage schema version must be upgraded from {} to {}",
                        schema_version, SCHEMA_VERSION
                    );
                };
                Ok(())
            })
            .boxed_env()
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
    .boxed_env()
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
                    E::set_storage(PROFILE_STORAGE_KEY, Some(&profile))
                }
                _ => E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
            }
        })
        .and_then(|_| E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&2)))
        .boxed_env()
}

fn migrate_storage_schema_to_v3<E: Env>() -> TryEnvFuture<()> {
    E::get_storage::<serde_json::Value>(PROFILE_STORAGE_KEY)
        .and_then(|mut profile| {
            match profile
                .as_mut()
                .and_then(|profile| profile.as_object_mut())
                .and_then(|profile| profile.get_mut("settings"))
                .and_then(|settings| settings.as_object_mut())
            {
                Some(settings) => {
                    settings.insert(
                        "streamingServerWarningDismissed".to_owned(),
                        serde_json::Value::Null,
                    );
                    E::set_storage(PROFILE_STORAGE_KEY, Some(&profile))
                }
                _ => E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
            }
        })
        .and_then(|_| E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&3)))
        .boxed_env()
}

fn migrate_storage_schema_to_v4<E: Env>() -> TryEnvFuture<()> {
    E::get_storage::<serde_json::Value>(PROFILE_STORAGE_KEY)
        .and_then(|mut profile| {
            match profile
                .as_mut()
                .and_then(|profile| profile.as_object_mut())
                .and_then(|profile| profile.get_mut("settings"))
                .and_then(|settings| settings.as_object_mut())
            {
                Some(settings) => {
                    settings.insert(
                        "seekTimeDuration".to_owned(),
                        serde_json::Value::Number(serde_json::Number::from(20000)),
                    );
                    E::set_storage(PROFILE_STORAGE_KEY, Some(&profile))
                }
                _ => E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
            }
        })
        .and_then(|_| E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&4)))
        .boxed_env()
}

fn migrate_storage_schema_to_v5<E: Env>() -> TryEnvFuture<()> {
    E::get_storage::<serde_json::Value>(PROFILE_STORAGE_KEY)
        .and_then(|mut profile| {
            match profile
                .as_mut()
                .and_then(|profile| profile.as_object_mut())
                .and_then(|profile| profile.get_mut("settings"))
                .and_then(|settings| settings.as_object_mut())
            {
                Some(settings) => {
                    settings.insert(
                        "audioLanguage".to_owned(),
                        serde_json::Value::String("eng".to_owned()),
                    );
                    settings.insert(
                        "audioPassthrough".to_owned(),
                        serde_json::Value::Bool(false),
                    );
                    E::set_storage(PROFILE_STORAGE_KEY, Some(&profile))
                }
                _ => E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
            }
        })
        .and_then(|_| E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&5)))
        .boxed_env()
}

fn migrate_storage_schema_to_v6<E: Env>() -> TryEnvFuture<()> {
    E::get_storage::<serde_json::Value>(PROFILE_STORAGE_KEY)
        .and_then(|mut profile| {
            match profile
                .as_mut()
                .and_then(|profile| profile.as_object_mut())
                .and_then(|profile| profile.get_mut("settings"))
                .and_then(|settings| settings.as_object_mut())
            {
                Some(settings) => {
                    let player_type = match settings.remove("playInExternalPlayer") {
                        Some(play_in_external_player) if play_in_external_player == true => {
                            serde_json::Value::String("external".to_owned())
                        }
                        _ => serde_json::Value::Null,
                    };
                    settings.insert("playerType".to_owned(), player_type);
                    settings.insert(
                        "autoFrameRateMatching".to_owned(),
                        serde_json::Value::Bool(false),
                    );
                    settings.insert(
                        "nextVideoNotificationDuration".to_owned(),
                        serde_json::Value::Number(serde_json::Number::from(35000)),
                    );
                    E::set_storage(PROFILE_STORAGE_KEY, Some(&profile))
                }
                _ => E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
            }
        })
        .and_then(|_| E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&6)))
        .boxed_env()
}

fn migrate_storage_schema_to_v7<E: Env>() -> TryEnvFuture<()> {
    E::get_storage::<serde_json::Value>(PROFILE_STORAGE_KEY)
        .and_then(|mut profile| {
            match profile
                .as_mut()
                .and_then(|profile| profile.as_object_mut())
                .and_then(|profile| profile.get_mut("settings"))
                .and_then(|settings| settings.as_object_mut())
            {
                Some(settings) => {
                    settings.remove("autoFrameRateMatching");
                    settings.insert(
                        "frameRateMatchingStrategy".to_owned(),
                        serde_json::Value::String("FrameRateOnly".to_owned()),
                    );
                    E::set_storage(PROFILE_STORAGE_KEY, Some(&profile))
                }
                _ => E::set_storage::<()>(PROFILE_STORAGE_KEY, None),
            }
        })
        .and_then(|_| E::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&7)))
        .boxed_env()
}

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use crate::constants::SCHEMA_VERSION;
    use crate::runtime::env::{migrate_storage_schema_to_v6, migrate_storage_schema_to_v7};
    use crate::{
        constants::{PROFILE_STORAGE_KEY, SCHEMA_VERSION_STORAGE_KEY},
        runtime::Env,
        unit_tests::{TestEnv, STORAGE},
    };

    fn set_profile_and_schema_version(profile: &Value, schema_v: u32) {
        let mut storage = STORAGE.write().expect("Should lock");

        let no_schema = storage.insert(SCHEMA_VERSION_STORAGE_KEY.into(), schema_v.to_string());
        assert!(
            no_schema.is_none(),
            "Current schema should be empty for this test"
        );

        let no_profile = storage.insert(PROFILE_STORAGE_KEY.into(), profile.to_string());
        assert!(
            no_profile.is_none(),
            "Current profile should be empty for this test"
        );
    }

    #[tokio::test]
    async fn test_migration_to_latest_version() {
        {
            let _test_env_guard = TestEnv::reset().expect("Should lock TestEnv");
            let profile_before = json!({
                "settings": {}
            });

            // setup storage for migration
            set_profile_and_schema_version(&profile_before, 1);

            // migrate storage
            TestEnv::migrate_storage_schema()
                .await
                .expect("Should migrate");

            let storage = STORAGE.read().expect("Should lock");

            assert_eq!(
                &SCHEMA_VERSION.to_string(),
                storage
                    .get(SCHEMA_VERSION_STORAGE_KEY)
                    .expect("Should have the schema set"),
                "Scheme version should now be updated"
            );
        }
    }

    #[tokio::test]
    async fn test_migration_from_5_to_6() {
        // Profile with external player (true)
        {
            let _test_env_guard = TestEnv::reset().expect("Should lock TestEnv");
            let profile_before = json!({
                "settings": {
                    "playInExternalPlayer": true,
                }
            });

            let migrated_profile = json!({
                "settings": {
                    "playerType": "external",
                    "autoFrameRateMatching": false,
                    "nextVideoNotificationDuration": 35000_u64
                }
            });

            // setup storage for migration
            set_profile_and_schema_version(&profile_before, 5);

            // migrate storage
            migrate_storage_schema_to_v6::<TestEnv>()
                .await
                .expect("Should migrate");

            let storage = STORAGE.read().expect("Should lock");

            assert_eq!(
                &6.to_string(),
                storage
                    .get(SCHEMA_VERSION_STORAGE_KEY)
                    .expect("Should have the schema set"),
                "Scheme version should now be updated"
            );
            assert_eq!(
                &migrated_profile.to_string(),
                storage
                    .get(PROFILE_STORAGE_KEY)
                    .expect("Should have the profile set"),
                "Profile should match"
            );
        }

        // Profile without external player (false)
        {
            let _test_env_guard = TestEnv::reset().expect("Should lock TestEnv");
            let profile_before = json!({
                "settings": {
                    "playInExternalPlayer": false,
                }
            });

            let migrated_profile = json!({
                "settings": {
                    "playerType": null,
                    "autoFrameRateMatching": false,
                    "nextVideoNotificationDuration": 35000_u64
                }
            });

            // setup storage for migration
            set_profile_and_schema_version(&profile_before, 5);

            // migrate storage
            migrate_storage_schema_to_v6::<TestEnv>()
                .await
                .expect("Should migrate");

            let storage = STORAGE.read().expect("Should lock");

            assert_eq!(
                &6.to_string(),
                storage
                    .get(SCHEMA_VERSION_STORAGE_KEY)
                    .expect("Should have the schema set"),
                "Scheme version should now be updated"
            );
            assert_eq!(
                &migrated_profile.to_string(),
                storage
                    .get(PROFILE_STORAGE_KEY)
                    .expect("Should have the profile set"),
                "Profile should match"
            );
        }

        // Profile with no external player set (null)
        {
            let _test_env_guard = TestEnv::reset().expect("Should lock TestEnv");
            let profile_before = json!({
                "settings": {}
            });

            let migrated_profile = json!({
                "settings": {
                    "playerType": null,
                    "autoFrameRateMatching": false,
                    "nextVideoNotificationDuration": 35000_u64
                }
            });

            // setup storage for migration
            set_profile_and_schema_version(&profile_before, 5);

            // migrate storage
            migrate_storage_schema_to_v6::<TestEnv>()
                .await
                .expect("Should migrate");

            let storage = STORAGE.read().expect("Should lock");

            assert_eq!(
                &6.to_string(),
                storage
                    .get(SCHEMA_VERSION_STORAGE_KEY)
                    .expect("Should have the schema set"),
                "Scheme version should now be updated"
            );
            assert_eq!(
                &migrated_profile.to_string(),
                storage
                    .get(PROFILE_STORAGE_KEY)
                    .expect("Should have the profile set"),
                "Profile should match"
            );
        }
    }

    #[tokio::test]
    async fn test_migration_from_6_to_7() {
        {
            let _test_env_guard = TestEnv::reset().expect("Should lock TestEnv");
            let profile_before = json!({
                "settings": {
                    "autoFrameRateMatching": false,
                }
            });

            let migrated_profile = json!({
                "settings": {
                    "frameRateMatchingStrategy": "FrameRateOnly"
                }
            });

            // setup storage for migration
            set_profile_and_schema_version(&profile_before, 6);

            // migrate storage
            migrate_storage_schema_to_v7::<TestEnv>()
                .await
                .expect("Should migrate");

            let storage = STORAGE.read().expect("Should lock");

            assert_eq!(
                &7.to_string(),
                storage
                    .get(SCHEMA_VERSION_STORAGE_KEY)
                    .expect("Should have the schema set"),
                "Scheme version should now be updated"
            );
            assert_eq!(
                &migrated_profile.to_string(),
                storage
                    .get(PROFILE_STORAGE_KEY)
                    .expect("Should have the profile set"),
                "Profile should match"
            );
        }

        // Profile without autoFrameRateMatching
        {
            let _test_env_guard = TestEnv::reset().expect("Should lock TestEnv");
            let profile_before = json!({
                "settings": {}
            });

            let migrated_profile = json!({
                "settings": {
                     "frameRateMatchingStrategy": "FrameRateOnly"
                }
            });

            // setup storage for migration
            set_profile_and_schema_version(&profile_before, 6);

            // migrate storage
            migrate_storage_schema_to_v7::<TestEnv>()
                .await
                .expect("Should migrate");

            let storage = STORAGE.read().expect("Should lock");

            assert_eq!(
                &7.to_string(),
                storage
                    .get(SCHEMA_VERSION_STORAGE_KEY)
                    .expect("Should have the schema set"),
                "Scheme version should now be updated"
            );
            assert_eq!(
                &migrated_profile.to_string(),
                storage
                    .get(PROFILE_STORAGE_KEY)
                    .expect("Should have the profile set"),
                "Profile should match"
            );
        }
    }
}
