pub mod types;
use self::types::*;

pub mod state_types;
use self::state_types::*;

#[cfg(test)]
mod tests {
    use serde_json::to_string;
    use reqwest::{Result,get};
    use super::*;
    use futures::Future;
    use futures::future;


#[test]
    fn it_works() {
        // @TODO: build a pipe of 
        // -> UserMiddleware -> CatalogMiddleware -> DetailMiddleware -> AddonsMiddleware ->
        // PlayerMiddleware -> LibNotifMiddleware -> join(discoverContainer, boardContainer, ...)
        let mut container = Container::with_reducer(CatalogGrouped::empty(), &catalogs_reducer);
        let fut = get_addons_async("https://api.strem.io/addonsofficialcollection.json")
            .and_then(|addons_resp| {
                 for addon in addons_resp.iter() {
                    for cat in addon.manifest.catalogs.iter() {
                        container.dispatch(&match get_catalogs(&addon, &cat.type_name, &cat.id) {
                            Ok(resp) => { Action::CatalogReceived(Ok(resp)) },
                            Err(_) => { Action::CatalogReceived(Err(())) },
                        });
                    }
                }
                // @TODO figure out how to do middlewares/reducers pipeline
                assert_eq!(container.get_state().groups.len(), 8);

                // @TODO move this out; testing is_supported
                let cinemeta_m = &addons_resp[0].manifest;
                assert_eq!(cinemeta_m.is_supported("meta".to_string(), "movie".to_string(), "tt0234".to_string()), true);
                assert_eq!(cinemeta_m.is_supported("meta".to_string(), "movie".to_string(), "somethingElse".to_string()), false);
                assert_eq!(cinemeta_m.is_supported("stream".to_string(), "movie".to_string(), "tt0234".to_string()), false);
                future::ok(())
            });
        fut.wait().expect("get_addons_async ran fine");
    }

    fn get_addons_async(url: &'static str) -> impl Future<Item=Vec<AddonDescriptor>, Error=()> {
        match get_addons(url) {
            Err(_) => future::err(()),
            Ok(e) => future::ok(e),
        }
    }
    fn get_addons(url: &'static str) -> reqwest::Result<Vec<AddonDescriptor>> {
        Ok(reqwest::get(url)?.json()?)
    }
    fn get_catalogs(addon: &AddonDescriptor, catalog_type: &String, id: &String) -> reqwest::Result<CatalogResponse> {
        let url = addon.transport_url.replace("/manifest.json", &format!("/catalog/{}/{}.json", catalog_type, id));
        Ok(reqwest::get(&url)?.json()?)
    }

    #[test]
    fn middlewares() {
        // to make sure we can't use 'static
        t_middlewares();
    }
    fn t_middlewares() {
        struct UserMiddleware{
            id: usize,
            user: Option<String>,
        }
        impl Handler for UserMiddleware {
            fn handle(&self, action: &Action, emit: &DispatcherFn) {
                println!("middleware {:?} received: {:?}", self.id, &action);
                emit(&Action::Open);
            }
        }

        // @TODO test what happens with no handlers

        // use Environment (immutable ref) in the Handlers 
        // construct reducers and final emit
        let chain = Chain::new(vec![
            Box::new(UserMiddleware{ id: 1, user: None }),
            Box::new(UserMiddleware{ id: 2, user: None }),
        ], Box::new(|action| {
            println!("final output {:?}", &action);
        }));

        // this is the dispatch operation
        let action = &Action::Init;
        chain.dispatch(action);
    }
}
