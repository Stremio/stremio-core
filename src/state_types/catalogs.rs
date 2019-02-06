use super::actions::*;
use crate::types::*;
use serde_derive::*;

const MAX_ITEMS: usize = 25;

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

pub fn catalogs_reducer(state: &CatalogGrouped, action: &Action) -> Option<Box<CatalogGrouped>> {
    match action {
        Action::LoadWithAddons(addons, load_action @ ActionLoad::CatalogGrouped) => {
            if let Some(aggr_req) = load_action.addon_aggr_req() {
                let groups = aggr_req
                    .plan(&addons)
                    .iter()
                    .map(|req| (req.to_owned(), Loadable::Loading))
                    .collect();
                Some(Box::new(CatalogGrouped { groups }))
            } else {
                None
            }
        }
        Action::AddonResponse(req, result) => {
            if let Some(idx) = state.groups.iter().position(|g| &g.0 == req) {
                // @TODO: this copy here is probably expensive; is there a way around it?
                // if there is, we should NOT touch state_container.rs, since it provides a good
                // conceptual basis
                // instead, we can either enclose internal fields in Rc<> or Cow<>
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
                Some(Box::new(CatalogGrouped { groups }))
            } else {
                None
            }
        }
        _ => {
            // Doesn't mutate
            None
        }
    }
}
