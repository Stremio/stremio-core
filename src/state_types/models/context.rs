use crate::types::addons::Descriptor;
use crate::types::api::{AuthKey, User};
use crate::state_types::{Update, Effects, Effect, Msg, Action};
use lazy_static::*;
use serde_derive::*;

lazy_static! {
    static ref DEFAULT_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(include_bytes!("../../../stremio-official-addons/index.json"))
            .expect("official addons JSON parse");
}

// These will be stored, so they need to implement both Serialize and Deserilaize
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Auth {
    key: AuthKey,
    pub user: User,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CtxContent {
    pub auth: Option<Auth>,
    pub addons: Vec<Descriptor>,
}
impl Default for CtxContent {
    fn default() -> Self {
        CtxContent {
            auth: None,
            addons: DEFAULT_ADDONS.to_owned(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Ctx {
    pub content: CtxContent,
    // Whether it's loaded from storage
    pub is_loaded: bool,
}

impl Update for Ctx {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::LoadCtx) => {
                Effects::one(load_storage_effect()).unchanged()
            }
            _ => Effects::none().unchanged()
        }
    }
}

// @TODO move these load/save?
const USER_DATA_KEY: &str = "userData";
fn load_storage_effect() -> Effect {
    unimplemented!()
}
