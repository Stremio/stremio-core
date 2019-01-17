use serde_derive::*;
use super::actions::*;
use crate::types::*;

// @TODO this might be needed outside of here
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Loadable<L, R, M> {
    NotLoaded,
    Loading(L),
    Ready(R),
    Message(M),
}

// @TODO better type for RequestId, Message
pub type RequestId = String;
pub type Message = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogGrouped {
    pub groups: Vec<Loadable<RequestId, CatalogResponse, Message>>
}
impl CatalogGrouped {
    pub fn empty() -> CatalogGrouped {
        CatalogGrouped{
            groups: vec![],
        }
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
            groups.push(Loadable::Loading(req_id.to_owned()));
            return Some(Box::new(CatalogGrouped{ groups }));
        },
        Action::CatalogReceived(req_id, result) => {
            // @TODO find a more elegant way to do this
            match state.groups.iter().position(|g| match g {
                Loadable::Loading(r) => req_id == r,
                _ => false,
            }) {
                Some(idx) => {
                    let mut groups = state.groups.to_owned();
                    groups[idx] = match result {
                        Ok(resp) => Loadable::Ready(resp.to_owned()),
                        Err(e) => Loadable::Message(e.to_owned()),
                    };
                    return Some(Box::new(CatalogGrouped{ groups }));
                },
                None => { return None },
            };
        },
        _ => {},
    };
    // Doesn't mutate
    None
}
