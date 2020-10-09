use crate::env::WebEnv;
use crate::model::WebModel;
use futures::{future, StreamExt};
use lazy_static::lazy_static;
use std::sync::RwLock;
use stremio_core::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY,
};
use stremio_core::models::common::Loadable;
use stremio_core::runtime::{Env, EnvError, Runtime};
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::profile::Profile;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

lazy_static! {
    static ref RUNTIME: RwLock<Option<Loadable<Runtime<WebEnv, WebModel>, EnvError>>> =
        Default::default();
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub async fn initialize_runtime(emit: js_sys::Function) {
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
                    WebEnv::exec(rx.for_each(move |msg| {
                        emit.call1(&JsValue::NULL, &JsValue::from_serde(&msg).unwrap())
                            .expect("emit event failed");
                        future::ready(())
                    }));
                    *RUNTIME.write().expect("runtime write failed") =
                        Some(Loadable::Ready(runtime));
                }
                Err(error) => {
                    *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Err(error));
                }
            }
        }
        Err(error) => {
            *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Err(error));
        }
    };
}

#[wasm_bindgen]
pub fn get_state(field: &JsValue) -> JsValue {
    match &*RUNTIME.read().expect("runtime read failed") {
        Some(Loadable::Ready(runtime)) => {
            let model = runtime.model().expect("model read failed");
            match field.into_serde() {
                Ok(field) => model.get_state(&field),
                Err(_) => JsValue::NULL,
            }
        }
        _ => panic!("runtime is not ready"),
    }
}

#[wasm_bindgen]
pub fn dispatch(action: &JsValue, field: &JsValue) -> JsValue {
    match &*RUNTIME.read().expect("runtime read failed") {
        Some(Loadable::Ready(runtime)) => match (action.into_serde(), field.into_serde()) {
            (Ok(action), Ok(field)) => {
                runtime.dispatch_to_field(&field, action);
                JsValue::TRUE
            }
            (Ok(action), Err(_)) => {
                runtime.dispatch(action);
                JsValue::TRUE
            }
            _ => JsValue::FALSE,
        },
        _ => panic!("runtime is not ready"),
    }
}
