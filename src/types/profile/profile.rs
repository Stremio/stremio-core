use crate::constants::OFFICIAL_ADDONS;
use crate::runtime::Env;
use crate::types::addon::Descriptor;
use crate::types::profile::{Auth, AuthKey, Settings};
use crate::types::{UniqueVec, UniqueVecAdapter};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

/// User ID
///
/// `Some` when authenticated
/// `None` if not authenticated
pub type UID = Option<String>;

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Profile {
    pub auth: Option<Auth>,
    #[serde_as(deserialize_as = "UniqueVec<Vec<_>, DescriptorUniqueVecAdapter>")]
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
    pub fn has_trakt<E: Env>(&self) -> bool {
        self.auth
            .as_ref()
            .and_then(|auth| auth.user.trakt.as_ref())
            .map(|trakt| E::now() < trakt.created_at + trakt.expires_in)
            .unwrap_or_default()
    }
}

struct DescriptorUniqueVecAdapter;

impl UniqueVecAdapter for DescriptorUniqueVecAdapter {
    type Input = Descriptor;
    type Output = Url;
    fn hash(descriptor: &Descriptor) -> Url {
        descriptor.transport_url.to_owned()
    }
}
