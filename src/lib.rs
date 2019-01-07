pub mod types;
use self::types::*;

pub mod state_types;
use self::state_types::*;
#[cfg(test)]
mod tests {
    use serde_json::to_string;
    use reqwest::{Result,get};
    use super::*;
    #[test]
    fn it_works() {
        // @TODO: build a pipe of 
        // -> UserMiddleware -> CatalogMiddleware -> DetailMiddleware -> AddonsMiddleware ->
        // PlayerMiddleware -> LibNotifMiddleware -> join(discoverContainer, boardContainer, ...)
        let mut container = StateContainer::with_reducer(
            CatalogGrouped{ items: vec![] },
            &|state, action| {
            match action {
                Action::CatalogsReceived(Ok(resp)) => {
                    // @TODO remove this; this is temporary
                    if resp.metas.len() != 100 {
                        return None
                    }
                    return Some(Box::new(CatalogGrouped{
                        items: resp.metas.to_owned()
                    }));
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
        });
        let addons_resp = get_addons("https://api.strem.io/addonsofficialcollection.json").unwrap();
        for addon in addons_resp.iter() {
            for cat in addon.manifest.catalogs.iter() {
                container.dispatch(&match get_catalogs(&addon, &cat.catalog_type, &cat.id) {
                    Ok(resp) => { Action::CatalogsReceived(Ok(resp)) },
                    Err(_) => { Action::CatalogsReceived(Err("request error")) },
                });
            }
        }
        // @TODO figure out how to do middlewares/reducers pipeline
        assert_eq!(
            match &container.get_state().items {
                // @TODO mathc on enums once we have them
                x => x.len(),
                _ => 0,
            },
            100,
        );
    }

    fn get_addons(url: &'static str) -> reqwest::Result<Vec<AddonDescriptor>> {
        Ok(reqwest::get(url)?.json()?)
    }
    fn get_catalogs(addon: &AddonDescriptor, catalog_type: &String, id: &String) -> reqwest::Result<CatalogResponse> {
        let url = addon.transport_url.replace("/manifest.json", &format!("/catalog/{}/{}.json", catalog_type, id));
        Ok(reqwest::get(&url)?.json()?)
    }

    fn get_watchhub() -> reqwest::Result<StreamResponse> {
        Ok(reqwest::get("https://watchhub-us.strem.io/stream/movie/tt0120338.json")?.json()?)
    }
}
