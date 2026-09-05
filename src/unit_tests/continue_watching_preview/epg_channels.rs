use chrono::{TimeZone, Utc};
use stremio_derive::Model;
use url::Url;

use crate::{
    models::{continue_watching_preview::ContinueWatchingPreview, ctx::Ctx},
    runtime::{
        msg::{Action, ActionCtx},
        Runtime, RuntimeAction,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestBehaviorHints},
        library::{LibraryBucket, LibraryItem, LibraryItemState},
        profile::Profile,
    },
    unit_tests::{TestEnv, NOW},
};

fn library_item(id: &str, r#type: &str) -> LibraryItem {
    LibraryItem {
        id: id.to_owned(),
        name: id.to_owned(),
        r#type: r#type.to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: Utc.with_ymd_and_hms(2026, 7, 2, 11, 0, 0).unwrap(),
        state: LibraryItemState {
            // in continue watching: time_offset > 0
            time_offset: 300_000,
            duration: 3_600_000,
            ..Default::default()
        },
        behavior_hints: Default::default(),
    }
}

fn epg_addon() -> Descriptor {
    Descriptor {
        transport_url: Url::parse("https://addon/manifest.json").unwrap(),
        flags: Default::default(),
        manifest: Manifest {
            id: "addon".to_owned(),
            types: vec!["tv".into()],
            id_prefixes: Some(vec!["pure:".to_owned()]),
            behavior_hints: ManifestBehaviorHints {
                epg_provider: true,
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

#[test]
fn continue_watching_excludes_epg_channels() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        continue_watching_preview: ContinueWatchingPreview,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 7, 2, 12, 0, 0).unwrap();

    let profile = Profile {
        addons: vec![epg_addon()],
        ..Default::default()
    };
    let library = LibraryBucket {
        uid: None,
        items: vec![
            ("tt123456".into(), library_item("tt123456", "movie")),
            ("pure:axn".into(), library_item("pure:axn", "tv")),
        ]
        .into_iter()
        .collect(),
    };

    let (continue_watching_preview, _) =
        ContinueWatchingPreview::new(&library, &Default::default(), &profile);
    assert_eq!(
        continue_watching_preview
            .items
            .iter()
            .map(|item| item.library_item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["tt123456"],
        "channels of epgProvider addons (matched by idPrefixes) should be excluded"
    );

    // uninstalling the addon stops the exclusion - the model
    // must recompute on ProfileChanged
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile,
                library,
                ..Default::default()
            },
            continue_watching_preview,
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UninstallAddon(epg_addon())),
        });
    });

    let continue_watching_preview = &runtime.model().unwrap().continue_watching_preview;
    assert_eq!(
        continue_watching_preview.items.len(),
        2,
        "the channel should reappear after the epgProvider addon is uninstalled"
    );
}
