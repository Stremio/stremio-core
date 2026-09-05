use crate::constants::OFFICIAL_ADDONS;
use crate::runtime::Env;
use crate::types::addon::Descriptor;
use crate::types::profile::{Auth, AuthKey, Settings};
use crate::types::{UniqueVec, UniqueVecAdapter};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

use super::UserId;

/// User ID
///
/// `Some` when authenticated
/// `None` if not authenticated
pub type UID = Option<UserId>;

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub auth: Option<Auth>,
    #[serde_as(deserialize_as = "UniqueVec<Vec<_>, DescriptorUniqueVecAdapter>")]
    pub addons: Vec<Descriptor>,
    /// This locking flag is raised when the API addon fetch request has failed
    /// in order to avoid overwriting the user's addons in the API
    /// if they install a new addon locally when we have defaulted to the official ones
    #[serde(default)]
    pub addons_locked: bool,
    pub settings: Settings,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            auth: None,
            addons: OFFICIAL_ADDONS.to_owned(),
            addons_locked: false,
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

    /// check whether the user has Trakt authentication token
    /// will return `false` if the token has expired
    pub fn has_trakt<E: Env>(&self) -> bool {
        self.auth
            .as_ref()
            .and_then(|auth| auth.user.trakt.as_ref())
            .map(|trakt| E::now() < trakt.created_at + trakt.expires_in)
            .unwrap_or_default()
    }

    /// Whether the id belongs to a live TV channel of an installed
    /// `epgProvider` addon - matched against the addons' `idPrefixes`
    pub fn is_epg_channel_id(&self, id: &str) -> bool {
        self.addons
            .iter()
            .filter(|addon| addon.manifest.behavior_hints.epg_provider)
            .flat_map(|addon| addon.manifest.id_prefixes.iter().flatten())
            .any(|id_prefix| id.starts_with(id_prefix))
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
