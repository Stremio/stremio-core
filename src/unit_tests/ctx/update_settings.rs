use crate::constants::PROFILE_STORAGE_KEY;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx};
use crate::state_types::Runtime;
use crate::types::profile::{Profile, Settings};
use crate::unit_tests::{Env, REQUESTS, STORAGE};
use stremio_derive::Model;

#[test]
fn actionctx_updatesettings() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let settings = Settings {
        subtitles_language: "bg".to_string(),
        subtitles_size: 150,
        ..Settings::default()
    };
    Env::reset();
    let (runtime, _rx) = Runtime::<Env, Model>::new(Model::default(), 1000);
    Env::run(|| runtime.dispatch(Action::Ctx(ActionCtx::UpdateSettings(settings.to_owned()))));
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
                serde_json::from_str::<Profile>(&data).unwrap().settings == settings
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
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
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
    Env::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    let (runtime, _rx) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                ..Default::default()
            },
        },
        1000,
    );
    Env::run(|| runtime.dispatch(Action::Ctx(ActionCtx::UpdateSettings(settings.to_owned()))));
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
                serde_json::from_str::<Profile>(&data).unwrap().settings == settings
            }),
        "Settings not updated in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
