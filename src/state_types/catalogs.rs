use super::actions::*;
use crate::types::addons::*;
use crate::state_types::Container;
use crate::types::MetaPreview;
use serde_derive::*;
use std::sync::Arc;

const MAX_ITEMS: usize = 25;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, M> {
    NotLoaded,
    Loading,
    ReadyEmpty,
    Ready(R),
    Message(M),
}
impl<R, M> Loadable<R, M> {
    pub fn is_ready(&self) -> bool {
        match self {
            Loadable::Ready(_) => true,
            _ => false,
        }
    }
}

// @TODO better type for Message
pub type Message = String;

type Group = Arc<(ResourceRequest, Loadable<Vec<MetaPreview>, Message>)>;
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CatalogGrouped {
    pub groups: Vec<Group>,
}
impl CatalogGrouped {
    pub fn new() -> CatalogGrouped {
        CatalogGrouped { groups: vec![] }
    }
}
impl Container for CatalogGrouped {
    fn dispatch(&self, action: &Action) -> Option<Box<Self>> {
        catalogs_reducer(&self, action)
    }
}

fn catalogs_reducer(state: &CatalogGrouped, action: &Action) -> Option<Box<CatalogGrouped>> {
    match action {
        Action::LoadWithCtx(
            Context { addons, .. },
            load_action @ ActionLoad::CatalogGrouped { .. },
        ) => {
            if let Some(aggr_req) = load_action.addon_aggr_req() {
                let groups = aggr_req
                    .plan(&addons)
                    .iter()
                    .map(|req| Arc::new((req.to_owned(), Loadable::Loading)))
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
                    Ok(ResourceResponse::Metas { metas }) if metas.len() == 0 => {
                        Loadable::ReadyEmpty
                    }
                    Ok(ResourceResponse::Metas { metas }) => {
                        Loadable::Ready(metas.iter().take(MAX_ITEMS).cloned().collect())
                    }
                    Ok(_) => Loadable::Message("unexpected ResourceResponse".to_owned()),
                    Err(e) => Loadable::Message(e.to_owned()),
                };
                groups[idx] = Arc::new((req.to_owned(), group_content));
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
