pub mod middlewares;
pub mod state_types;
pub mod types;

#[cfg(test)]
mod tests {
    use self::middlewares::*;
    use self::state_types::*;
    use self::types::*;
    use super::*;
    use futures::{future, Future};
    use serde::de::DeserializeOwned;
    use std::error::Error;
    use std::marker::PhantomData;

    #[test]
    fn it_works() {
        // @TODO: build a pipe of
        // -> UserMiddleware -> CatalogMiddleware -> DetailMiddleware -> AddonsMiddleware ->
        // PlayerMiddleware -> LibNotifMiddleware -> join(discoverContainer, boardContainer, ...)
        let mut container = Container::with_reducer(CatalogGrouped::new_empty(), &catalogs_reducer);
        let addons_resp = get_addons("https://api.strem.io/addonsofficialcollection.json").unwrap();
        for addon in addons_resp.iter() {
            for cat in addon.manifest.catalogs.iter() {
                let req_id = format!("{}/{}/{}", &addon.manifest.id, &cat.type_name, &cat.id);
                container.dispatch(&Action::CatalogRequested(req_id.to_owned()));
                container.dispatch(&match get_catalogs(&addon, &cat.type_name, &cat.id) {
                    Ok(resp) => Action::CatalogReceived(req_id, Ok(resp)),
                    Err(e) => Action::CatalogReceived(req_id, Err(e.description().to_owned())),
                });
            }
        }
        // @TODO figure out how to do middlewares/reducers pipeline
        assert_eq!(container.get_state().groups.len(), 9);

        // @TODO move this out; testing is_supported
        let cinemeta_m = &addons_resp[0].manifest;
        assert_eq!(
            cinemeta_m.is_supported(
                "meta".to_string(),
                "movie".to_string(),
                "tt0234".to_string()
            ),
            true
        );
        assert_eq!(
            cinemeta_m.is_supported(
                "meta".to_string(),
                "movie".to_string(),
                "somethingElse".to_string()
            ),
            false
        );
        assert_eq!(
            cinemeta_m.is_supported(
                "stream".to_string(),
                "movie".to_string(),
                "tt0234".to_string()
            ),
            false
        );
    }

    fn get_addons(url: &'static str) -> reqwest::Result<Vec<AddonDescriptor>> {
        Ok(reqwest::get(url)?.json()?)
    }
    fn get_catalogs(
        addon: &AddonDescriptor,
        catalog_type: &String,
        id: &String,
    ) -> reqwest::Result<CatalogResponse> {
        let url = addon.transport_url.replace(
            "/manifest.json",
            &format!("/catalog/{}/{}.json", catalog_type, id),
        );
        Ok(reqwest::get(&url)?.json()?)
    }

    #[test]
    fn middlewares() {
        // to make sure we can't use 'static
        t_middlewares();
    }
    fn t_middlewares() {
        // @TODO: assert if this works
        // @TODO test what happens with no handlers

        let chain = Chain::new(
            vec![
                Box::new(UserMiddleware::<Env> {
                    user: None,
                    env: PhantomData,
                }),
                Box::new(CatalogMiddleware::<Env> { env: PhantomData }),
                // @TODO: reducers multiplexer middleware
            ],
            Box::new(|action| {
                println!("final output {:?}", &action);
            }),
        );

        // this is the dispatch operation
        let action = &Action::Init;
        chain.dispatch(action);
    }

    struct Env;
    impl Environment for Env {
        fn fetch_serde<T: 'static>(url: String) -> Box<Future<Item = Box<T>, Error = Box<Error>>>
        where
            T: DeserializeOwned,
        {
            Box::new(match reqwest::get(&url) {
                Err(e) => future::err(e.into()),
                Ok(mut resp) => match resp.json() {
                    Err(e) => future::err(e.into()),
                    Ok(resp) => future::ok(Box::new(resp)),
                },
            })
        }
        fn exec(fut: Box<Future<Item = (), Error = ()>>) {
            fut.wait().unwrap();
        }
    }
}
