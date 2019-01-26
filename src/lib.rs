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
    use serde::Serialize;
    use std::error::Error;

    #[test]
    fn it_works() {
        // @TODO: build a pipe of
        // -> UserMiddleware -> CatalogMiddleware -> DetailMiddleware -> AddonsMiddleware ->
        // PlayerMiddleware -> LibNotifMiddleware -> join(discoverContainer, boardContainer, ...)
        let mut container = Container::with_reducer(CatalogGrouped::new(), &catalogs_reducer);
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
        assert_eq!(cinemeta_m.is_supported("meta", "movie", "tt0234"), true);
        assert_eq!(
            cinemeta_m.is_supported("meta", "movie", "somethingElse"),
            false
        );
        assert_eq!(cinemeta_m.is_supported("stream", "movie", "tt0234"), false);
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
        let container = std::rc::Rc::new(std::cell::RefCell::new(Container::with_reducer(
            CatalogGrouped::new(),
            &catalogs_reducer,
        )));
        let container_ref = container.clone();
        let chain = Chain::new(
            vec![
                Box::new(UserMiddleware::<Env>::new()),
                Box::new(CatalogMiddleware::<Env>::new()),
                Box::new(ContainerHandler::new(0, container)),
                // @TODO: reducers multiplexer middleware
            ],
            Box::new(move |action| {
                if let Action::NewState(_) = action {
                    println!("new state {:?}", container_ref.borrow().get_state());
                }
            }),
        );

        // this is the dispatch operation
        let action = &Action::Init;
        chain.dispatch(action);
    }

    struct Env;
    impl Environment for Env {
        fn fetch_serde<IN, OUT>(request: &Request<IN>) -> EnvFuture<Box<OUT>>
        where
            IN: 'static + Serialize,
            OUT: 'static + DeserializeOwned,
        {
            // @TODO method
            // @TODO headers
            // @TODO body
            /*
            // Can't work for now, as it needs + Send
            let client = reqwest::r#async::Client::new();
            let fut = client.get(&request.uri().to_string())
                .send()
                .and_then(|mut res: reqwest::r#async::Response| {
                    res.json::<OUT>()
                })
                .map(|res| Box::new(res))
                .map_err(|e| e.into());
            Box::new(fut)
            */
            Box::new(match reqwest::get(&request.uri().to_string()) {
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
        fn get_storage<T: 'static + DeserializeOwned>(_key: &str) -> EnvFuture<Option<Box<T>>> {
            Box::new(future::err("unimplemented".into()))
        }
        fn set_storage<T: 'static + Serialize>(_key: &str, _value: &T) -> EnvFuture<()> {
            Box::new(future::err("unimplemented".into()))
        }
    }
}
