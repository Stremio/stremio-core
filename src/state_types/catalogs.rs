use super::actions::*;
use crate::types::*;
use serde_derive::*;

const MAX_ITEMS: usize = 25;

// @TODO this might be needed outside of here
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, M> {
    NotLoaded,
    Loading,
    Ready(R),
    Message(M),
}

// @TODO better type for Message
pub type Message = String;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CatalogGrouped {
    pub groups: Vec<(ResourceRequest, Loadable<CatalogResponse, Message>)>,
}
impl CatalogGrouped {
    pub fn new() -> CatalogGrouped {
        CatalogGrouped { groups: vec![] }
    }
}

// @TODO if we want to make this generic, we have to make MetaItem/LibItem/NotifItem implement the
// same trait
// the event CatalogsReceived must be generic too
pub fn catalogs_reducer(state: &CatalogGrouped, action: &Action) -> Option<Box<CatalogGrouped>> {
    match action {
        Action::LoadWithAddons(addons, load_action @ ActionLoad::CatalogGrouped) => {
            if let Some(aggr_req) = load_action.addon_aggr_req() {
                let groups = aggr_req
                    .plan(&addons)
                    .iter()
                    .map(|req| (req.to_owned(), Loadable::Loading))
                    .collect();
                return Some(Box::new(CatalogGrouped { groups }));
            }
        }
        Action::AddonResponse(req, result) => {
            if let Some(idx) = state.groups.iter().position(|g| &g.0 == req) {
                // @TODO: is there a way to do this without copying all groups
                let mut groups = state.groups.to_owned();
                groups[idx].1 = match result {
                    Ok(resp) => Loadable::Ready(CatalogResponse {
                        metas: resp
                            .metas
                            .iter()
                            .take(MAX_ITEMS)
                            .map(|m| m.to_owned())
                            .collect(),
                    }),
                    Err(e) => Loadable::Message(e.to_owned()),
                };
                return Some(Box::new(CatalogGrouped { groups }));
            }
        }
        _ => {}
    };
    // Doesn't mutate
    None
}
