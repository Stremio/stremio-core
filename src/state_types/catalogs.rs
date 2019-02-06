use super::actions::*;
use crate::types::*;
use serde_derive::*;
use std::rc::Rc;

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

// @TODO separate type for group
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CatalogGrouped {
    pub groups: Vec<Rc<(ResourceRequest, Loadable<Vec<MetaPreview>, Message>)>>,
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
                    .map(|req| Rc::new((req.to_owned(), Loadable::Loading)))
                    .collect();
                Some(Box::new(CatalogGrouped { groups }))
            } else {
                None
            }
        }
        Action::AddonResponse(req, result) => {
            if let Some(idx) = state.groups.iter().position(|g| &g.0 == req) {
                let mut groups = state.groups.to_owned();
                let group_content = match result {
                    Ok(ResourceResponse::Metas(metas)) => Loadable::Ready(
                        metas
                            .iter()
                            .take(MAX_ITEMS)
                            .map(|m| m.to_owned())
                            .collect()
                    ),
                    Ok(_) => Loadable::Message("unexpected response kind".to_owned()),
                    Err(e) => Loadable::Message(e.to_owned()),
                };
                groups[idx] = Rc::new((req.to_owned(), group_content));
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
