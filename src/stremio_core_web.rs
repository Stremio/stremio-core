use crate::{env::WebEnv, event::WebEvent, model::WebModel};

use std::sync::RwLock;

use enclose::enclose;
use futures::{future, FutureExt, StreamExt};
use lazy_static::lazy_static;
use tracing::{info, Level};

use stremio_core::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY, STREAMS_STORAGE_KEY,
};
use stremio_core::models::common::Loadable;
use stremio_core::runtime::msg::Action;
use stremio_core::runtime::{Env, EnvError, Runtime, RuntimeAction, RuntimeEvent};
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::profile::Profile;
use stremio_core::types::resource::Stream;
use stremio_core::types::streams::StreamsBucket;

use tracing_wasm::WASMLayerConfigBuilder;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

lazy_static! {
    static ref RUNTIME: RwLock<Option<Loadable<Runtime<WebEnv, WebModel>, EnvError>>> =
        Default::default();
}

#[wasm_bindgen(start)]
pub fn start() {
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    // This is not needed for tracing_wasm to work, but it is a common tool for getting proper error line numbers for panics.
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    let max_level = Level::TRACE;
    #[cfg(not(debug_assertions))]
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
            let storage_result = future::try_join4(
                WebEnv::get_storage::<Profile>(PROFILE_STORAGE_KEY),
                WebEnv::get_storage::<LibraryBucket>(LIBRARY_RECENT_STORAGE_KEY),
                WebEnv::get_storage::<LibraryBucket>(LIBRARY_STORAGE_KEY),
                WebEnv::get_storage::<StreamsBucket>(STREAMS_STORAGE_KEY),
            )
            .await;
            match storage_result {
                Ok((profile, recent_bucket, other_bucket, streams_bucket)) => {
                    let profile = profile.unwrap_or_default();
                    let mut library = LibraryBucket::new(profile.uid(), vec![]);
                    if let Some(recent_bucket) = recent_bucket {
                        library.merge_bucket(recent_bucket);
                    };
                    if let Some(other_bucket) = other_bucket {
                        library.merge_bucket(other_bucket);
                    };
                    let streams_bucket = streams_bucket.unwrap_or_default();
                    let (model, effects) = WebModel::new(profile, library, streams_bucket);
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
                            .call1(&JsValue::NULL, &JsValue::from_serde(&event).unwrap())
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
                    Err(JsValue::from_serde(&error).unwrap())
                }
            }
        }
        Err(error) => {
            *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Err(error.to_owned()));
            Err(JsValue::from_serde(&error).unwrap())
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
    JsValue::from_serde(&*model).unwrap()
}

#[wasm_bindgen]
pub fn get_state(field: JsValue) -> JsValue {
    let field = field.into_serde().expect("get state failed");
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
    let action = action.into_serde::<Action>().expect("dispatch failed");
    let field = field.into_serde().expect("dispatch failed");
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready")
        .as_ref()
        .expect("runtime is not ready");
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
    let event = event.into_serde().expect("analytics failed");
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready")
        .as_ref()
        .expect("runtime is not ready");
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
        Some(Ok(stream)) => JsValue::from_serde(&stream).unwrap(),
        _ => JsValue::NULL,
    }
}
