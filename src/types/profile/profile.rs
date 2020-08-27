use crate::constants::OFFICIAL_ADDONS;
use crate::types::addon::Descriptor;
use crate::types::profile::{Auth, Settings};
use serde::{Deserialize, Serialize};

pub type UID = Option<String>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub auth: Option<Auth>,
    pub addons: Vec<Descriptor>,
    pub settings: Settings,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            auth: None,
            addons: OFFICIAL_ADDONS.to_owned(),
            settings: Settings::default(),
        }
    }
}

impl Profile {
    pub fn uid(&self) -> UID {
        self.auth.as_ref().map(|auth| auth.user.id.to_owned())
    }
}
