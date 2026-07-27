mod mark_video_as_watched;
mod next_stream;

use crate::{
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad},
        Runtime, RuntimeAction, RuntimeEvent,
    },
    types::resource::{Stream, StreamBehaviorHints, StreamSource},
    unit_tests::{TestEnv, EVENTS},
};
use std::sync::{Arc, RwLock};
use stremio_derive::Model;

#[test]
fn identical_player_loads_are_observable() {
    #[derive(Model, Default, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, rx) = Runtime::<TestEnv, _>::new(TestModel::default(), vec![], 1000);
    let runtime = Arc::new(RwLock::new(runtime));
    let selected = Selected {
        stream: Stream {
            source: StreamSource::Url {
                url: "https://source_url".parse().unwrap(),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        },
        stream_request: None,
        meta_request: None,
        subtitles_path: None,
    };

    TestEnv::run_with_runtime(rx, runtime.clone(), move || {
        let runtime = runtime.read().unwrap();
        for _ in 0..2 {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::Player(Box::new(selected.clone()))),
            });
        }
    });

    let events = EVENTS.read().unwrap();
    let new_states = events
        .iter()
        .filter_map(|event| {
            match event
                .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
                .unwrap()
            {
                RuntimeEvent::NewState(fields, _) => Some(fields),
                RuntimeEvent::CoreEvent(_) => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(new_states.len(), 2);
    for fields in new_states {
        assert_eq!(fields.as_slice(), [TestModelField::Player]);
    }
}
