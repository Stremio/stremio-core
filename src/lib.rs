use env_web::Env;
use futures::future;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::panic;
use stremio_core::state_types::messages::{Action, Msg};
use stremio_core::state_types::models::{
    CatalogFiltered, CatalogsWithExtra, ContinueWatching, Ctx, LibraryFiltered, MetaDetails,
    SelectablePriority, StreamingServerSettingsModel,
};
use stremio_core::state_types::{Environment, Runtime, Update, UpdateWithCtx};
use stremio_core::types::addons::DescriptorPreview;
use stremio_core::types::MetaPreview;
use stremio_derive::Model;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
extern crate console_error_panic_hook;

#[derive(Model, Serialize)]
pub struct Model {
    ctx: Ctx<Env>,
    continue_watching: ContinueWatching,
    board: CatalogsWithExtra,
    discover: CatalogFiltered<MetaPreview>,
    library: LibraryFiltered,
    search: CatalogsWithExtra,
    meta_details: MetaDetails,
    addons: CatalogFiltered<DescriptorPreview>,
    streaming_server_settings: StreamingServerSettingsModel,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelField {
    Ctx,
    ContinueWatching,
    Board,
    Discover,
    Library,
    Search,
    MetaDetails,
    Addons,
    StreamingServerSettings,
}

#[wasm_bindgen]
pub struct StremioCoreWeb {
    runtime: Runtime<Env, Model>,
}

#[wasm_bindgen]
impl StremioCoreWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(emit: js_sys::Function) -> StremioCoreWeb {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let app = Model {
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
            addons: CatalogFiltered {
                selectable: Default::default(),
                catalog_resource: Default::default(),
                selectable_priority: SelectablePriority::Catalog,
            },
            streaming_server_settings: Default::default(),
        };
        let (runtime, rx) = Runtime::<Env, Model>::new(app, 1000);
        Env::exec(Box::new(rx.for_each(move |msg| {
            let _ = emit.call1(&JsValue::NULL, &JsValue::from_serde(&msg).unwrap());
            future::ok(())
        })));
        StremioCoreWeb { runtime }
    }

    pub fn dispatch(&self, action: &JsValue, model_field: &JsValue) {
        if let Ok(action) = action.into_serde::<Action>() {
            let message = Msg::Action(action);
            let effects = if let Ok(model_field) = model_field.into_serde::<ModelField>() {
                self.runtime.dispatch_with(|model| match model_field {
                    ModelField::Ctx => model.ctx.update(&message),
                    ModelField::ContinueWatching => {
                        model.continue_watching.update(&model.ctx, &message)
                    }
                    ModelField::Board => model.board.update(&model.ctx, &message),
                    ModelField::Discover => model.discover.update(&model.ctx, &message),
                    ModelField::Library => model.library.update(&model.ctx, &message),
                    ModelField::Search => model.search.update(&model.ctx, &message),
                    ModelField::MetaDetails => model.meta_details.update(&model.ctx, &message),
                    ModelField::Addons => model.addons.update(&model.ctx, &message),
                    ModelField::StreamingServerSettings => {
                        model.streaming_server_settings.update(&model.ctx, &message)
                    }
                })
            } else {
                self.runtime.dispatch(&message)
            };
            Env::exec(effects);
        };
    }

    pub fn get_state(&self, model_field: &JsValue) -> JsValue {
        let model = &*self.runtime.app.read().unwrap();
        if let Ok(model_field) = model_field.into_serde::<ModelField>() {
            match model_field {
                ModelField::Ctx => JsValue::from_serde(&model.ctx).unwrap(),
                ModelField::ContinueWatching => {
                    JsValue::from_serde(&model.continue_watching).unwrap()
                }
                ModelField::Board => JsValue::from_serde(&model.board).unwrap(),
                ModelField::Discover => JsValue::from_serde(&model.discover).unwrap(),
                ModelField::Library => JsValue::from_serde(&model.library).unwrap(),
                ModelField::Search => JsValue::from_serde(&model.search).unwrap(),
                ModelField::MetaDetails => JsValue::from_serde(&model.meta_details).unwrap(),
                ModelField::Addons => JsValue::from_serde(&model.addons).unwrap(),
                ModelField::StreamingServerSettings => {
                    JsValue::from_serde(&model.streaming_server_settings).unwrap()
                }
            }
        } else {
            JsValue::from_serde(model).unwrap()
        }
    }
}
