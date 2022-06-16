use crate::constants::OFFICIAL_ADDONS;
use crate::types::addon::Descriptor;
use crate::types::profile::{Auth, AuthKey, Settings};
use crate::types::{UniqueVec, UniqueVecAdapter};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

pub type UID = Option<String>;

#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Profile {
    pub auth: Option<Auth>,
    #[serde_as(as = "UniqueVec<Descriptor, Url, DescriptorUniqueVecAdapter>")]
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
    pub fn auth_key(&self) -> Option<&AuthKey> {
        self.auth.as_ref().map(|auth| &auth.key)
    }
}

struct DescriptorUniqueVecAdapter;

impl UniqueVecAdapter<Descriptor, Url> for DescriptorUniqueVecAdapter {
    fn hash(value: &Descriptor) -> Url {
        value.transport_url.to_owned()
    }
}
