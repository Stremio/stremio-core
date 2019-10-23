use futures::future;
use serde::{Deserialize, Serialize};
use stremio_core::state_types::*;
// required to make stremio_derive work :(
use env_web::*;
use futures::stream::Stream;
pub use stremio_core::state_types;
use stremio_core::types::addons::Descriptor;
use stremio_core::types::MetaPreview;
use stremio_derive::*;
use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;
use std::panic;

#[derive(Model, Default, Serialize)]
pub struct Model {
    ctx: Ctx<Env>,
    recent: LibRecent,
    board: CatalogGrouped,
    discover: CatalogFiltered<MetaPreview>,
    addons: CatalogFiltered<Descriptor>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "model", content = "args")]
pub enum WebAction {
    Discover(Action),
    Addons(Action),
    All(Action),
}

#[wasm_bindgen]
pub struct ContainerService {
    runtime: Runtime<Env, Model>,
}

#[wasm_bindgen]
impl ContainerService {
    #[wasm_bindgen(constructor)]
    pub fn new(emit: js_sys::Function) -> ContainerService {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let app = Model::default();
        let (runtime, rx) = Runtime::<Env, Model>::new(app, 1000);
        Env::exec(Box::new(rx.for_each(move |msg| {
            let _ = emit.call1(&JsValue::NULL, &JsValue::from_serde(&msg).unwrap());
            future::ok(())
        })));
        ContainerService { runtime }
    }

    pub fn dispatch(&self, action_js: &JsValue) {
        let action: WebAction = action_js
            .into_serde()
            .expect("WebAction could not be deserialized");
        let effects = match action {
            WebAction::Discover(action) => self
                .runtime
                .dispatch_with(|model| model.discover.update(&model.ctx, &Msg::Action(action))),
            WebAction::Addons(action) => self
                .runtime
                .dispatch_with(|model| model.addons.update(&model.ctx, &Msg::Action(action))),
            WebAction::All(action) => self.runtime.dispatch(&Msg::Action(action)),
        };
        Env::exec(effects);
    }

    pub fn get_state(&self) -> JsValue {
        JsValue::from_serde(&*self.runtime.app.read().unwrap()).unwrap()
    }
}
