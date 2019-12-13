use crate::app_model::{AppModel, ModelFieldName};
use env_web::Env;
use futures::future;
use futures::stream::Stream;
use std::panic;
use stremio_core::state_types::messages::{Action, Msg};
use stremio_core::state_types::models::{CatalogFiltered, SelectablePriority};
use stremio_core::state_types::{Environment, Runtime, Update, UpdateWithCtx};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
extern crate console_error_panic_hook;

#[wasm_bindgen]
pub struct StremioCoreWeb {
    runtime: Runtime<Env, AppModel>,
}

#[wasm_bindgen]
impl StremioCoreWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(emit: js_sys::Function) -> StremioCoreWeb {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let app = AppModel {
            ctx: Default::default(),
            continue_watching: Default::default(),
            board: Default::default(),
            discover: CatalogFiltered {
                selectable: Default::default(),
                catalog_resource: Default::default(),
                selectable_priority: SelectablePriority::Type,
            },
            library: Default::default(),
            search: Default::default(),
            meta_details: Default::default(),
            addon_details: Default::default(),
            addons: CatalogFiltered {
                selectable: Default::default(),
                catalog_resource: Default::default(),
                selectable_priority: SelectablePriority::Catalog,
            },
            streaming_server_settings: Default::default(),
            library_items: Default::default(),
        };
        let (runtime, rx) = Runtime::<Env, AppModel>::new(app, 1000);
        Env::exec(Box::new(rx.for_each(move |msg| {
            let _ = emit.call1(&JsValue::NULL, &JsValue::from_serde(&msg).unwrap());
            future::ok(())
        })));
        StremioCoreWeb { runtime }
    }

    pub fn dispatch(&self, action: &JsValue, model_field: &JsValue) -> JsValue {
        if let Ok(action) = action.into_serde::<Action>() {
            let message = Msg::Action(action);
            let effects = if let Ok(model_field) = model_field.into_serde::<ModelFieldName>() {
                self.runtime.dispatch_with(|model| match model_field {
                    ModelFieldName::Ctx => model.ctx.update(&message),
                    ModelFieldName::ContinueWatching => {
                        model.continue_watching.update(&model.ctx, &message)
                    }
                    ModelFieldName::Board => model.board.update(&model.ctx, &message),
                    ModelFieldName::Discover => model.discover.update(&model.ctx, &message),
                    ModelFieldName::Library => model.library.update(&model.ctx, &message),
                    ModelFieldName::Search => model.search.update(&model.ctx, &message),
                    ModelFieldName::MetaDetails => model.meta_details.update(&model.ctx, &message),
                    ModelFieldName::AddonDetails => {
                        model.addon_details.update(&model.ctx, &message)
                    }
                    ModelFieldName::Addons => model.addons.update(&model.ctx, &message),
                    ModelFieldName::StreamingServerSettings => {
                        model.streaming_server_settings.update(&model.ctx, &message)
                    }
                    ModelFieldName::LibraryItems => {
                        model.library_items.update(&model.ctx, &message)
                    }
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
                ModelFieldName::ContinueWatching => {
                    JsValue::from_serde(&model.continue_watching).unwrap()
                }
                ModelFieldName::Board => JsValue::from_serde(&model.board).unwrap(),
                ModelFieldName::Discover => JsValue::from_serde(&model.discover).unwrap(),
                ModelFieldName::Library => JsValue::from_serde(&model.library).unwrap(),
                ModelFieldName::Search => JsValue::from_serde(&model.search).unwrap(),
                ModelFieldName::MetaDetails => JsValue::from_serde(&model.meta_details).unwrap(),
                ModelFieldName::AddonDetails => JsValue::from_serde(&model.meta_details).unwrap(),
                ModelFieldName::Addons => JsValue::from_serde(&model.addons).unwrap(),
                ModelFieldName::StreamingServerSettings => {
                    JsValue::from_serde(&model.streaming_server_settings).unwrap()
                }
                ModelFieldName::LibraryItems => JsValue::from_serde(&model.library_items).unwrap(),
            }
        } else {
            JsValue::from_serde(model).unwrap()
        }
    }
}
