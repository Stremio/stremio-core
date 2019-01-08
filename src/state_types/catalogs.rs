use serde_derive::*;
use super::actions::*;
use crate::types::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogGrouped<T> {
    pub groups: Vec<Vec<T>>
}
impl<T> CatalogGrouped<T> {
    pub fn empty() -> CatalogGrouped<T> {
        CatalogGrouped{
            groups: vec![],
        }
    }
}

// @TODO if we want to make this generic, we have to make MetaItem/LibItem/NotifItem implement the
// same trait
// the event CatalogsReceived must be generic too
pub fn catalogs_reducer(state: &CatalogGrouped<MetaItem>, action: &Action) -> Option<Box<CatalogGrouped<MetaItem>>> {
    match action {
        Action::CatalogsReceived(Ok(resp)) => {
            // @TODO ordering
            let mut new_groups = state.groups.to_owned();
            new_groups.push(resp.metas.to_owned());
            return Some(Box::new(CatalogGrouped{ groups: new_groups }));
        },
        // @TODO
        Action::CatalogsReceived(Err(err)) => {
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
