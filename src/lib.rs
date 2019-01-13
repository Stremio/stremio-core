pub mod types;
use self::types::*;

pub mod state_types;
use self::state_types::*;

#[cfg(test)]
mod tests {
    use serde_json::{to_string,from_value};
    use reqwest::{Result,get};
    use super::*;
    use futures::{Future,future};
    use std::error::Error;
    use std::rc::Rc;
    use std::marker::PhantomData;
    use serde::de::DeserializeOwned;

#[test]
    fn it_works() {
        // @TODO: build a pipe of 
        // -> UserMiddleware -> CatalogMiddleware -> DetailMiddleware -> AddonsMiddleware ->
        // PlayerMiddleware -> LibNotifMiddleware -> join(discoverContainer, boardContainer, ...)
        let mut container = Container::with_reducer(CatalogGrouped::empty(), &catalogs_reducer);
        let addons_resp = get_addons("https://api.strem.io/addonsofficialcollection.json").unwrap();
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
        // @TODO move this
        struct UserMiddleware<T: Environment>{
            id: usize,
            user: Option<String>,
            env: PhantomData<T>,
        }
        impl<T> Handler for UserMiddleware<T> where T: Environment {
            fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
                emit(&Action::Open);
                let fut = T::fetch_serde::<Vec<AddonDescriptor>>("https://api.strem.io/addonscollection.json".to_owned())
                    .and_then(move |addons| {
                        emit(&Action::AddonsLoaded(addons));
                        future::ok(())
                    });
                // @TODO error handling on the future, do not call .wait here
                fut.wait().expect("got addons");
            }
        }

        // @TODO test what happens with no handlers

        // use Environment (immutable ref) in the Handlers 
        // construct reducers and final emit
        let chain = Chain::new(vec![
            Box::new(UserMiddleware::<Env>{ id: 1, user: None, env: PhantomData }),
            Box::new(UserMiddleware::<Env>{ id: 2, user: None, env: PhantomData }),
        ], Box::new(|action| {
            println!("final output {:?}", &action);
        }));

        // this is the dispatch operation
        let action = &Action::Init;
        chain.dispatch(action);
    }

    struct Env();
    impl Environment for Env {
        fn fetch_serde<T: 'static>(url: String) -> Box<Future<Item=Box<T>, Error=Box<Error>>> where T: DeserializeOwned {
            Box::new(match reqwest::get(&url) {
                Err(e) => future::err(e.into()),
                Ok(mut resp) => {
                    match resp.json() {
                        Err(e) => future::err(e.into()),
                        Ok(resp) => future::ok(Box::new(resp)),
                    }
                }
            })
        }
    }
}
