use crate::analytics::Analytics;
use crate::env::WebEnv;
use crate::event::WebEvent;
use crate::model::WebModel;
use futures::{future, StreamExt};
use lazy_static::lazy_static;
use serde_json::json;
use std::sync::RwLock;
use stremio_core::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY,
};
use stremio_core::models::common::Loadable;
use stremio_core::runtime::msg::Event;
use stremio_core::runtime::{Env, EnvError, Runtime, RuntimeAction, RuntimeEvent};
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::profile::Profile;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};

lazy_static! {
    static ref ANALYTICS: Analytics<WebEnv> = Default::default();
    static ref RUNTIME: RwLock<Option<Loadable<Runtime<WebEnv, WebModel>, EnvError>>> =
        Default::default();
}

pub fn emit_to_analytics(event: WebEvent) {
    match &*RUNTIME.read().expect("runtime read failed") {
        Some(Loadable::Ready(runtime)) => {
            let model = runtime.model().expect("model read failed");
            let event = match event {
                WebEvent::CoreEvent(Event::UserAuthenticated { .. }) => json!({
                    "name": "login",
                }),
                _ => return,
            };
            ANALYTICS.emit(event, &model.ctx);
        }
        _ => panic!("runtime is not ready"),
    };
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let closure = Closure::wrap(Box::new(|| {
        ANALYTICS.flush_next_batch();
    }) as Box<dyn FnMut()>);
    web_sys::window()
        .expect("window is not available")
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            30 * 1000,
        )
        .expect("set_interval failed");
    closure.forget();
}

#[wasm_bindgen]
pub async fn initialize_runtime(emit_to_js: js_sys::Function) -> Result<(), JsValue> {
    if RUNTIME.read().expect("runtime read failed").is_some() {
        panic!("unable to initialize runtime multiple times");
    };

    *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Loading);
    let migration_result = WebEnv::migrate_storage_schema().await;
    match migration_result {
        Ok(_) => {
            let storage_result = future::try_join3(
                WebEnv::get_storage::<Profile>(PROFILE_STORAGE_KEY),
                WebEnv::get_storage::<LibraryBucket>(LIBRARY_RECENT_STORAGE_KEY),
                WebEnv::get_storage::<LibraryBucket>(LIBRARY_STORAGE_KEY),
            )
            .await;
            match storage_result {
                Ok((profile, recent_bucket, other_bucket)) => {
                    let profile = profile.unwrap_or_default();
                    let mut library = LibraryBucket::new(profile.uid(), vec![]);
                    if let Some(recent_bucket) = recent_bucket {
                        library.merge_bucket(recent_bucket);
                    };
                    if let Some(other_bucket) = other_bucket {
                        library.merge_bucket(other_bucket);
                    };
                    let (model, effects) = WebModel::new(profile, library);
                    let (runtime, rx) = Runtime::<WebEnv, _>::new(model, effects, 1000);
                    WebEnv::exec(rx.for_each(move |event| {
                        emit_to_js
                            .call1(&JsValue::NULL, &JsValue::from_serde(&event).unwrap())
                            .expect("emit event failed");
                        if let RuntimeEvent::CoreEvent(event) = event {
                            emit_to_analytics(WebEvent::CoreEvent(event));
                        };
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
pub fn get_state(field: JsValue) -> JsValue {
    match field.into_serde() {
        Ok(field) => match &*RUNTIME.read().expect("runtime read failed") {
            Some(Loadable::Ready(runtime)) => runtime
                .model()
                .expect("model read failed")
                .get_state(&field),
            _ => panic!("runtime is not ready"),
        },
        Err(error) => panic!("get state failed: {}", error.to_string()),
    }
}

#[wasm_bindgen]
pub fn dispatch(action: JsValue, field: JsValue) {
    match (action.into_serde(), field.into_serde()) {
        (Ok(action), Ok(field)) => {
            match &*RUNTIME.read().expect("runtime read failed") {
                Some(Loadable::Ready(runtime)) => runtime.dispatch(RuntimeAction { action, field }),
                _ => panic!("runtime is not ready"),
            };
        }
        (Err(error), _) | (_, Err(error)) => {
            panic!("dispatch failed: {}", error.to_string());
        }
    };
}

#[wasm_bindgen]
pub fn analytics(event: JsValue) {
    match event.into_serde() {
        Ok(event) => emit_to_analytics(WebEvent::UIEvent(event)),
        Err(error) => panic!("emit failed: {}", error.to_string()),
    }
}
