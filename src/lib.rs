use futures::future;
use serde::{Serialize};
use stremio_core::state_types::*;
// required to make stremio_derive work :(
pub use stremio_core::state_types;
use stremio_core::types::MetaPreview;
use stremio_core::types::addons::DescriptorPreview;
use stremio_derive::*;
use wasm_bindgen::prelude::*;
use futures::stream::Stream;
use env_web::*;

#[derive(Model, Default, Serialize)]
pub struct Model {
    ctx: Ctx<Env>,
    recent: LibRecent,
    board: CatalogGrouped,
    discover: CatalogFiltered<MetaPreview>,
    addons: CatalogFiltered<DescriptorPreview>,
}

#[wasm_bindgen]
pub struct ContainerService {
    runtime: Runtime<Env, Model>,
}

#[wasm_bindgen]
impl ContainerService {
    #[wasm_bindgen(constructor)]
    pub fn new(emit: js_sys::Function) -> ContainerService {
        let app = Model::default();
        let (runtime, rx) = Runtime::<Env, Model>::new(app, 1000);
        Env::exec(Box::new(rx.for_each(move |msg| {
            let _ = emit.call1(&JsValue::NULL, &JsValue::from_serde(&msg).unwrap());
            future::ok(())
        })));
        ContainerService { runtime }
    }

    pub fn dispatch(&self, action_js: &JsValue) {
        let action: Action = action_js
            .into_serde()
            .expect("Action could not be deserialized");
        Env::exec(self.runtime.dispatch(&Msg::Action(action)));
    }

    pub fn get_state(&self) -> JsValue {
        JsValue::from_serde(&*self.runtime.app.read().unwrap()).unwrap()
    }
}
