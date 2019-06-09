use crate::types::addons::Descriptor;
use crate::types::api::{AuthKey, User};
use crate::state_types::{Update, Effects, Msg};
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
pub struct Ctx {
    pub auth: Option<Auth>,
    pub addons: Vec<Descriptor>,
    // Whether it's loaded from storage
    pub is_loaded: bool,
}

impl Default for Ctx {
    fn default() -> Self {
        Ctx {
            auth: None,
            addons: DEFAULT_ADDONS.to_owned(),
            is_loaded: false,
        }
    }
}

impl Update for Ctx {
    fn update(&mut self, _: &Msg) -> Effects {
        // @TODO
        Effects::none()
    }
}
