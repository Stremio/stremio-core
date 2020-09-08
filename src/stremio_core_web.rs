use crate::env::Env;
use crate::model::{WebModel, WebModelField};
use futures::{future, StreamExt};
use std::ops::Deref;
use stremio_core::runtime::{Environment, Runtime};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen]
pub struct StremioCoreWeb {
    runtime: Runtime<Env, WebModel>,
}

#[wasm_bindgen]
impl StremioCoreWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(emit: js_sys::Function) -> StremioCoreWeb {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        let (runtime, rx) = Runtime::<Env, _>::new(WebModel::default(), 1000);
        Env::exec(rx.for_each(move |msg| {
            emit.call1(&JsValue::NULL, &JsValue::from_serde(&msg).unwrap())
                .expect("emit event failed");
            future::ready(())
        }));
        StremioCoreWeb { runtime }
    }
    pub fn dispatch(&self, field: &JsValue, action: &JsValue) -> JsValue {
        match (field.into_serde(), action.into_serde()) {
            (Ok(field), Ok(action)) => {
                self.runtime.dispatch_to_field(&field, action);
                JsValue::from(true)
            }
            (Err(_), Ok(action)) => {
                self.runtime.dispatch(action);
                JsValue::from(true)
            }
            _ => JsValue::from(false),
        }
    }
    pub fn get_state(&self, model_field: &JsValue) -> JsValue {
        let model = self.runtime.model().expect("model read failed");
        if let Ok(model_field) = model_field.into_serde::<WebModelField>() {
            match model_field {
                WebModelField::Ctx => JsValue::from_serde(&model.ctx).unwrap(),
                WebModelField::ContinueWatchingPreview => {
                    JsValue::from_serde(&model.continue_watching_preview).unwrap()
                }
                WebModelField::Board => JsValue::from_serde(&model.board).unwrap(),
                WebModelField::Discover => JsValue::from_serde(&model.discover).unwrap(),
                WebModelField::Library => JsValue::from_serde(&model.library).unwrap(),
                WebModelField::ContinueWatching => {
                    JsValue::from_serde(&model.continue_watching).unwrap()
                }
                WebModelField::Search => JsValue::from_serde(&model.search).unwrap(),
                WebModelField::MetaDetails => JsValue::from_serde(&model.meta_details).unwrap(),
                WebModelField::Addons => JsValue::from_serde(&model.addons).unwrap(),
                WebModelField::AddonDetails => JsValue::from_serde(&model.addon_details).unwrap(),
                WebModelField::StreamingServer => {
                    JsValue::from_serde(&model.streaming_server).unwrap()
                }
                WebModelField::Player => JsValue::from_serde(&model.player).unwrap(),
            }
        } else {
            JsValue::from_serde(&model.deref()).unwrap()
        }
    }
}
