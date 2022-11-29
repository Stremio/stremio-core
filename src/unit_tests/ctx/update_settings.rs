use crate::constants::PROFILE_STORAGE_KEY;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Runtime, RuntimeAction};
use crate::types::profile::{Profile, Settings};
use crate::unit_tests::{TestEnv, REQUESTS, STORAGE};
use stremio_derive::Model;

#[test]
fn actionctx_updatesettings() {
    #[derive(Model, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let settings = Settings {
        subtitles_language: "bg".to_string(),
        subtitles_size: 150,
        ..Settings::default()
    };
    let _env_mutex = TestEnv::reset();
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(TestModel::default(), vec![], 1000);
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
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(data).unwrap().settings == settings
            }),
        "Settings updated successfully in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_updatesettings_not_changed() {
    #[derive(Model, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let settings = Settings {
        subtitles_language: "bg".to_string(),
        subtitles_size: 150,
        ..Settings::default()
    };
    let profile = Profile {
        settings: settings.to_owned(),
        ..Default::default()
    };
    let _env_mutex = TestEnv::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile,
                ..Default::default()
            },
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
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(data).unwrap().settings == settings
            }),
        "Settings not updated in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
