use std::sync::RwLock;

use enclose::enclose;
use futures::{future, try_join, FutureExt, StreamExt};
use gloo_utils::format::JsValueSerdeExt;
use lazy_static::lazy_static;
use tracing::{info, Level};
use tracing_wasm::WASMLayerConfigBuilder;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use stremio_core::{
    constants::{
        DISMISSED_EVENTS_STORAGE_KEY, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
        NOTIFICATIONS_STORAGE_KEY, PROFILE_STORAGE_KEY, SEARCH_HISTORY_STORAGE_KEY,
        STREAMS_STORAGE_KEY,
    },
    models::common::Loadable,
    runtime::{msg::Action, Env, EnvError, Runtime, RuntimeAction, RuntimeEvent},
    types::{
        events::DismissedEventsBucket, library::LibraryBucket, notifications::NotificationsBucket,
        profile::Profile, resource::Stream, search_history::SearchHistoryBucket,
        streams::StreamsBucket,
    },
};

use crate::{
    env::WebEnv,
    event::WebEvent,
    model::{WebModel, WebModelField},
};

lazy_static! {
    static ref RUNTIME: RwLock<Option<Loadable<Runtime<WebEnv, WebModel>, EnvError>>> =
        Default::default();
}

#[wasm_bindgen(start)]
pub fn start() {
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    // This is not needed for tracing_wasm to work, but it is a common tool for getting proper error line numbers for panics.
    console_error_panic_hook::set_once();

    #[cfg(any(debug_assertions, feature = "log-trace"))]
    let max_level = Level::TRACE;
    #[cfg(all(not(debug_assertions), not(feature = "log-trace")))]
    let max_level = Level::ERROR;

    let config = WASMLayerConfigBuilder::default()
        .set_max_level(max_level)
        .build();
    // setup wasm tracing Subscriber on web console
    tracing_wasm::set_as_global_default_with_config(config);

    info!(?max_level, "Logging level");
}

#[wasm_bindgen]
pub async fn initialize_runtime(emit_to_ui: js_sys::Function) -> Result<(), JsValue> {
    if RUNTIME.read().expect("runtime read failed").is_some() {
        panic!("runtime initialization has already started");
    };

    *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Loading);
    let env_init_result = WebEnv::init().await;
    match env_init_result {
        Ok(_) => {
            let storage_result = try_join!(
                WebEnv::get_storage::<Profile>(PROFILE_STORAGE_KEY),
                WebEnv::get_storage::<LibraryBucket>(LIBRARY_RECENT_STORAGE_KEY),
                WebEnv::get_storage::<LibraryBucket>(LIBRARY_STORAGE_KEY),
                WebEnv::get_storage::<StreamsBucket>(STREAMS_STORAGE_KEY),
                WebEnv::get_storage::<NotificationsBucket>(NOTIFICATIONS_STORAGE_KEY),
                WebEnv::get_storage::<SearchHistoryBucket>(SEARCH_HISTORY_STORAGE_KEY),
                WebEnv::get_storage::<DismissedEventsBucket>(DISMISSED_EVENTS_STORAGE_KEY),
            );
            match storage_result {
                Ok((
                    profile,
                    recent_bucket,
                    other_bucket,
                    streams_bucket,
                    notifications_bucket,
                    search_history_bucket,
                    dismissed_events_bucket,
                )) => {
                    let profile = profile.unwrap_or_default();
                    let mut library = LibraryBucket::new(profile.uid(), vec![]);
                    if let Some(recent_bucket) = recent_bucket {
                        library.merge_bucket(recent_bucket);
                    };
                    if let Some(other_bucket) = other_bucket {
                        library.merge_bucket(other_bucket);
                    };
                    let streams_bucket =
                        streams_bucket.unwrap_or_else(|| StreamsBucket::new(profile.uid()));
                    let notifications_bucket = notifications_bucket
                        .unwrap_or(NotificationsBucket::new::<WebEnv>(profile.uid(), vec![]));
                    let search_history_bucket =
                        search_history_bucket.unwrap_or(SearchHistoryBucket::new(profile.uid()));
                    let dismissed_events_bucket = dismissed_events_bucket
                        .unwrap_or(DismissedEventsBucket::new(profile.uid()));
                    let (model, effects) = WebModel::new(
                        profile,
                        library,
                        streams_bucket,
                        notifications_bucket,
                        search_history_bucket,
                        dismissed_events_bucket,
                    );
                    let (runtime, rx) = Runtime::<WebEnv, _>::new(
                        model,
                        effects.into_iter().collect::<Vec<_>>(),
                        1000,
                    );
                    WebEnv::exec_concurrent(rx.for_each(move |event| {
                        if let RuntimeEvent::CoreEvent(event) = &event {
                            WebEnv::exec_concurrent(WebEnv::get_location_hash().then(
                                enclose!((event) move |location_hash| async move {
                                    let runtime = RUNTIME.read().expect("runtime read failed");
                                    let runtime = runtime
                                        .as_ref()
                                        .expect("runtime is not ready")
                                        .as_ref()
                                        .expect("runtime is not ready");
                                    let model = runtime.model().expect("model read failed");
                                    let path = location_hash.split('#').last().map(|path| path.to_owned()).unwrap_or_default();
                                    WebEnv::emit_to_analytics(
                                        &WebEvent::CoreEvent(Box::new(event.to_owned())),
                                        &model,
                                        &path
                                    );
                                }),
                            ));
                        };
                        emit_to_ui
                            .call1(&JsValue::NULL, &<JsValue as JsValueSerdeExt>::from_serde(&event).expect("Event handler: JsValue from Event"))
                            .expect("emit event failed");
                        future::ready(())
                    }));
                    *RUNTIME.write().expect("runtime write failed") =
                        Some(Loadable::Ready(runtime));
                    Ok(())
                }
                Err(error) => {
                    *RUNTIME.write().expect("runtime write failed") =
                        Some(Loadable::Err(error.to_owned()));
                    Err(<JsValue as JsValueSerdeExt>::from_serde(&error)
                        .expect("Storage: JsValue from Event"))
                }
            }
        }
        Err(error) => {
            *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Err(error.to_owned()));
            Err(<JsValue as JsValueSerdeExt>::from_serde(&error).expect("JsValue from Event"))
        }
    }
}

#[wasm_bindgen]
#[cfg(debug_assertions)]
pub fn get_debug_state() -> JsValue {
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready")
        .as_ref()
        .expect("runtime is not ready");
    let model = runtime.model().expect("model read failed");
    <JsValue as JsValueSerdeExt>::from_serde(&*model).expect("JsValue from WebModel")
}

#[wasm_bindgen]
pub fn get_state(field: JsValue) -> JsValue {
    let field = JsValueSerdeExt::into_serde(&field).expect("get state failed");
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready")
        .as_ref()
        .expect("runtime is not ready");
    let model = runtime.model().expect("model read failed");
    model.get_state(&field)
}

#[wasm_bindgen]
pub fn dispatch(action: JsValue, field: JsValue, location_hash: JsValue) {
    let action: Action =
        JsValueSerdeExt::into_serde(&action).expect("dispatch failed because of Action");
    let field: Option<WebModelField> =
        JsValueSerdeExt::into_serde(&field).expect("dispatch failed because of Field");
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready - None")
        .as_ref()
        .expect("runtime is not ready - Loading or Error");
    {
        let model = runtime.model().expect("model read failed");
        let path = location_hash
            .as_string()
            .and_then(|location_hash| location_hash.split('#').last().map(|path| path.to_owned()))
            .unwrap_or_default();
        WebEnv::emit_to_analytics(
            &WebEvent::CoreAction(Box::new(action.to_owned())),
            &model,
            &path,
        );
    }
    runtime.dispatch(RuntimeAction { action, field });
}

#[wasm_bindgen]
pub fn analytics(event: JsValue, location_hash: JsValue) {
    let event =
        JsValueSerdeExt::into_serde(&event).expect("UIEvent deserialization for analytics failed");
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready - None")
        .as_ref()
        .expect("runtime is not ready - Loading or Error");
    let model = runtime.model().expect("model read failed");
    let path = location_hash
        .as_string()
        .and_then(|location_hash| location_hash.split('#').last().map(|path| path.to_owned()))
        .unwrap_or_default();
    WebEnv::emit_to_analytics(&WebEvent::UIEvent(event), &model, &path);
}

#[wasm_bindgen]
pub fn decode_stream(stream: JsValue) -> JsValue {
    let stream = stream.as_string().map(Stream::decode);
    match stream {
        Some(Ok(stream)) => {
            <JsValue as JsValueSerdeExt>::from_serde(&stream).expect("JsValue from Stream")
        }
        _ => JsValue::NULL,
    }
}
