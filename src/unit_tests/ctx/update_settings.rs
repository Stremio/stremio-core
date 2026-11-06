use crate::{
    constants::PROFILE_STORAGE_KEY,
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        Runtime, RuntimeAction,
    },
    types::{
        events::DismissedEventsBucket,
        library::LibraryBucket,
        notifications::NotificationsBucket,
        profile::{Profile, Settings},
        search_history::SearchHistoryBucket,
        server_urls::ServerUrlsBucket,
        streams::StreamsBucket,
    },
    unit_tests::{TestEnv, REQUESTS, STORAGE},
};
use stremio_derive::Model;
use url::Url;

#[test]
fn actionctx_updatesettings() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let settings = Settings {
        subtitles_language: Some("bg".to_string()),
        subtitles_size: 150,
        subtitles_language_filter_enabled: true,
        preferred_subtitle_language1: "fra".to_owned(),
        preferred_subtitle_language2: Some("jpn".to_owned()),
        ..Settings::default()
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(TestModel { ctx }, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UpdateSettings(settings.to_owned())),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile.settings,
        settings,
        "Settings updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .is_some_and(|data| {
                serde_json::from_str::<Profile>(data).unwrap().settings == settings
            }),
        "Settings updated successfully in storage"
    );
    let disabled_settings = Settings {
        subtitles_language_filter_enabled: false,
        ..settings.clone()
    };
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UpdateSettings(disabled_settings.clone())),
        })
    });
    let stored = STORAGE.read().unwrap();
    let reloaded: Profile = serde_json::from_str(stored.get(PROFILE_STORAGE_KEY).unwrap()).unwrap();
    assert_eq!(reloaded.settings, disabled_settings);
    assert_eq!(reloaded.settings.preferred_subtitle_language1, "fra");
    assert_eq!(
        reloaded.settings.preferred_subtitle_language2.as_deref(),
        Some("jpn")
    );
    assert_eq!(reloaded.settings.subtitles_language.as_deref(), Some("bg"));
    assert!(reloaded.settings.subtitles_auto_select);
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_updatesettings_not_changed() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let settings = Settings {
        subtitles_language: Some("bg".to_string()),
        subtitles_size: 150,
        ..Settings::default()
    };
    let profile = Profile {
        settings: settings.to_owned(),
        ..Default::default()
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                profile,
                LibraryBucket::default(),
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UpdateSettings(settings.to_owned())),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile.settings,
        settings,
        "Settings not updated in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .is_some_and(|data| {
                serde_json::from_str::<Profile>(data).unwrap().settings == settings
            }),
        "Settings not updated in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_updatesettings_invalid_streaming_server_url() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let settings = Settings {
        streaming_server_url: Url::parse("javascript:x").unwrap(),
        ..Settings::default()
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(TestModel { ctx }, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UpdateSettings(settings)),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile.settings,
        Settings::default(),
        "Settings not updated in memory"
    );
    assert!(
        STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).is_none(),
        "Settings not updated in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
