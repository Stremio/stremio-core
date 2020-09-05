use crate::app_model::{AppModel, ModelFieldName};
use crate::env::Env;
use core::pin::Pin;
use futures::{future, StreamExt};
use std::panic;
use stremio_core::state_types::msg::{Action, Msg};
use stremio_core::state_types::{Environment, Runtime, Update, UpdateWithCtx};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen]
pub struct StremioCoreWeb {
    runtime: Runtime<Env, AppModel>,
}

#[wasm_bindgen]
impl StremioCoreWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(emit: js_sys::Function) -> StremioCoreWeb {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let app = AppModel::default();
        let (runtime, rx) = Runtime::<Env, AppModel>::new(app, 1000);
        Env::exec(Pin::new(Box::new(rx.for_each(move |msg| {
            let _ = emit.call1(&JsValue::NULL, &JsValue::from_serde(&msg).unwrap());
            future::ready(())
        }))));
        StremioCoreWeb { runtime }
    }
    pub fn dispatch(&self, action: &JsValue, model_field: &JsValue) -> JsValue {
        if let Ok(action) = action.into_serde::<Action>() {
            let message = Msg::Action(action);
            let effects = if let Ok(model_field) = model_field.into_serde::<ModelFieldName>() {
                self.runtime.dispatch_with(|model| match model_field {
                    ModelFieldName::Ctx => model.ctx.update(&message),
                    ModelFieldName::ContinueWatchingPreview => {
                        model.continue_watching_preview.update(&model.ctx, &message)
                    }
                    ModelFieldName::Board => model.board.update(&model.ctx, &message),
                    ModelFieldName::Discover => model.discover.update(&model.ctx, &message),
                    ModelFieldName::Library => model.library.update(&model.ctx, &message),
                    ModelFieldName::ContinueWatching => {
                        model.continue_watching.update(&model.ctx, &message)
                    }
                    ModelFieldName::Search => model.search.update(&model.ctx, &message),
                    ModelFieldName::MetaDetails => model.meta_details.update(&model.ctx, &message),
                    ModelFieldName::Addons => model.addons.update(&model.ctx, &message),
                    ModelFieldName::AddonDetails => {
                        model.addon_details.update(&model.ctx, &message)
                    }
                    ModelFieldName::StreamingServer => {
                        model.streaming_server.update(&model.ctx, &message)
                    }
                    ModelFieldName::Player => model.player.update(&model.ctx, &message),
                })
            } else {
                self.runtime.dispatch(&message)
            };
            Env::exec(effects);
            JsValue::from(true)
        } else {
            JsValue::from(false)
        }
    }
    pub fn get_state(&self, model_field: &JsValue) -> JsValue {
        let model = &*self.runtime.app.read().unwrap();
        if let Ok(model_field) = model_field.into_serde::<ModelFieldName>() {
            match model_field {
                ModelFieldName::Ctx => JsValue::from_serde(&model.ctx).unwrap(),
                ModelFieldName::ContinueWatchingPreview => {
                    JsValue::from_serde(&model.continue_watching_preview).unwrap()
                }
                ModelFieldName::Board => JsValue::from_serde(&model.board).unwrap(),
                ModelFieldName::Discover => JsValue::from_serde(&model.discover).unwrap(),
                ModelFieldName::Library => JsValue::from_serde(&model.library).unwrap(),
                ModelFieldName::ContinueWatching => {
                    JsValue::from_serde(&model.continue_watching).unwrap()
                }
                ModelFieldName::Search => JsValue::from_serde(&model.search).unwrap(),
                ModelFieldName::MetaDetails => JsValue::from_serde(&model.meta_details).unwrap(),
                ModelFieldName::Addons => JsValue::from_serde(&model.addons).unwrap(),
                ModelFieldName::AddonDetails => JsValue::from_serde(&model.addon_details).unwrap(),
                ModelFieldName::StreamingServer => {
                    JsValue::from_serde(&model.streaming_server).unwrap()
                }
                ModelFieldName::Player => JsValue::from_serde(&model.player).unwrap(),
            }
        } else {
            JsValue::from_serde(model).unwrap()
        }
    }
}
