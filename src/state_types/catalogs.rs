use serde_derive::*;
use super::actions::*;
use crate::types::*;

// @TODO this might be needed outside of here
#[derive(Debug, Serialize)]
pub enum Loadable<T, M> {
    NotLoaded,
    Loading,
    Ready(T),
    Message(M),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogGrouped {
    // @TODO Loadable
    pub groups: Vec<CatalogResponse>
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
    match action {
        Action::CatalogReceived(Ok(resp)) => {
            // @TODO ordering
            let mut new_groups = state.groups.to_owned();
            new_groups.push(resp.to_owned());
            return Some(Box::new(CatalogGrouped{ groups: new_groups }));
        },
        // @TODO
        Action::CatalogReceived(Err(err)) => {
            return None
            //return Some(Box::new(State{
            //    catalog: Loadable::Message(err.to_string())
            //}));
        },
        _ => {},
    };
    // Doesn't mutate
    None
}
