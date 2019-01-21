use super::actions::*;
use crate::types::*;
use serde_derive::*;

const MAX_ITEMS: usize = 25;

// @TODO this might be needed outside of here
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag="type", content="content")]
pub enum Loadable<R, M> {
    NotLoaded,
    Loading,
    Ready(R),
    Message(M),
}

// @TODO struct Group, which would have req_id, the content (Loadable) and basic info about the
// add-on (name, version, maybe more)
// @TODO better type for RequestId, Message
pub type RequestId = String;
pub type Message = String;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CatalogGrouped {
    pub groups: Vec<(RequestId, Loadable<CatalogResponse, Message>)>,
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
    // @TODO: can we get rid of some of the to_owned's?
    // @TODO: can we make this more DRY
    match action {
        Action::CatalogRequested(req_id) => {
            let mut groups = state.groups.to_owned();
            groups.push((req_id.to_owned(), Loadable::Loading));
            return Some(Box::new(CatalogGrouped { groups }));
        }
        Action::CatalogReceived(req_id, result) => {
            if let Some(idx) = state.groups.iter().position(|g| &g.0 == req_id) {
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
            };
        }
        _ => {}
    };
    // Doesn't mutate
    None
}
