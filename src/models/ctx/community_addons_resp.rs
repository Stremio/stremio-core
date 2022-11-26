use crate::types::addon::Descriptor;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommunityAddonsResp {
    pub addons: Vec<Descriptor>,
}
