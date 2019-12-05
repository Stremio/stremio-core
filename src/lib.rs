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
use stremio_core::state_types::{Environment, Runtime, UpdateWithCtx};
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
#[serde(tag = "model", content = "args")]
pub enum WebAction {
    Board(Action),
    Discover(Action),
    Search(Action),
    Addons(Action),
    All(Action),
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

    pub fn dispatch(&self, action_js: &JsValue) {
        let action: WebAction = action_js
            .into_serde()
            .expect("WebAction could not be deserialized");
        let effects = match action {
            WebAction::Board(action) => self
                .runtime
                .dispatch_with(|model| model.board.update(&model.ctx, &Msg::Action(action))),
            WebAction::Discover(action) => self
                .runtime
                .dispatch_with(|model| model.discover.update(&model.ctx, &Msg::Action(action))),
            WebAction::Search(action) => self
                .runtime
                .dispatch_with(|model| model.search.update(&model.ctx, &Msg::Action(action))),
            WebAction::Addons(action) => self
                .runtime
                .dispatch_with(|model| model.addons.update(&model.ctx, &Msg::Action(action))),
            WebAction::All(action) => self.runtime.dispatch(&Msg::Action(action)),
        };
        Env::exec(effects);
    }

    pub fn get_state(&self, model_name: &JsValue) -> JsValue {
        let model_name = model_name.as_string();
        match model_name.as_ref().map(String::as_str) {
            Some("ctx") => JsValue::from_serde(&(*self.runtime.app.read().unwrap()).ctx).unwrap(),
            Some("continue_watching") => {
                JsValue::from_serde(&(*self.runtime.app.read().unwrap()).continue_watching).unwrap()
            }
            Some("board") => {
                JsValue::from_serde(&(*self.runtime.app.read().unwrap()).board).unwrap()
            }
            Some("discover") => {
                JsValue::from_serde(&(*self.runtime.app.read().unwrap()).discover).unwrap()
            }
            Some("library") => {
                JsValue::from_serde(&(*self.runtime.app.read().unwrap()).library).unwrap()
            }
            Some("search") => {
                JsValue::from_serde(&(*self.runtime.app.read().unwrap()).search).unwrap()
            }
            Some("meta_details") => {
                JsValue::from_serde(&(*self.runtime.app.read().unwrap()).meta_details).unwrap()
            }
            Some("addons") => {
                JsValue::from_serde(&(*self.runtime.app.read().unwrap()).addons).unwrap()
            }
            Some("streaming_server_settings") => {
                JsValue::from_serde(&(*self.runtime.app.read().unwrap()).streaming_server_settings)
                    .unwrap()
            }
            _ => JsValue::from_serde(&*self.runtime.app.read().unwrap()).unwrap(),
        }
    }
}
