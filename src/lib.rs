pub mod addon_transport;
pub mod state_types;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::addon_transport::*;
    use crate::state_types::*;
    use crate::types::addons::{Descriptor, ResourceRef, ResourceResponse};
    use futures::future::lazy;
    use futures::{future, Future};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use tokio::executor::current_thread::spawn;
    use tokio::runtime::current_thread::run;

    /*
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
    enum ContainerId {
        Board,
        Discover,
    }

    // @TODO
    #[test]
    fn middlewares() {
        // @TODO: Fix: the assumptions we are testing against are pretty much based on the current
        // official addons; e.g. assuming 6 groups, or 4 groups when searching
        let container = Rc::new(ContainerHolder::new(CatalogGrouped::new()));
        let container_filtered = Rc::new(ContainerHolder::new(CatalogFiltered::new()));
        let muxer = Rc::new(ContainerMuxer::new(
            vec![
                Box::new(ContextMiddleware::<Env>::new()),
                Box::new(AddonsMiddleware::<Env>::new()),
            ],
            vec![
                (
                    ContainerId::Board,
                    container.clone() as Rc<dyn ContainerInterface>,
                ),
                (
                    ContainerId::Discover,
                    container_filtered.clone() as Rc<dyn ContainerInterface>,
                ),
            ],
            Box::new(|_event| {
                //if let Event::NewState(_) = _event {
                //    dbg!(_event);
                //}
            }),
        ));

        run(lazy(enclose!((muxer) move || {
            // this is the dispatch operation
            let action = &Action::Load(ActionLoad::CatalogGrouped { extra: vec![] });
            muxer.dispatch(action);
            future::ok(())
        })));

        // since this is after the .run() has ended, it will be OK
        let state = container.get_state_owned();
        assert_eq!(state.groups.len(), 6, "groups is the right length");
        assert!(state.groups[0].1.is_ready());
        for g in state.groups.iter() {
            assert!(
                match g.1 {
                    Loadable::Ready(_) => true,
                    Loadable::Message(_) => true,
                    _ => false,
                },
                "group is Ready or Message"
            );
        }
        if !state.groups.iter().any(|g| g.1.is_ready()) {
            panic!("there are no items that are Ready {:?}", state);
        }

        // Now try the same, but with Search
        run(lazy(enclose!((muxer) move || {
            let extra = vec![("search".to_owned(), "grand tour".to_owned())];
            let action = &Action::Load(ActionLoad::CatalogGrouped { extra });
            muxer.dispatch(action);
            future::ok(())
        })));
        let state = container.get_state_owned();
        assert_eq!(
            state.groups.len(),
            4,
            "groups is the right length when searching"
        );

        let resource_req = ResourceRequest {
            base: "https://v3-cinemeta.strem.io/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "movie", "top"),
        };
        run(lazy(enclose!((muxer, resource_req) move || {
            muxer.dispatch_load_to(&ContainerId::Discover, &ActionLoad::CatalogFiltered { resource_req });
            future::ok(())
        })));
        let state = container_filtered.get_state_owned();
        assert_eq!(state.selected, Some(resource_req), "selected is right");
        assert_eq!(state.item_pages.len(), 1, "item_pages is the right length");
        assert!(state.item_pages[0].is_ready(), "first page is ready");

        run(lazy(enclose!((muxer, resource_req) move || {
            muxer.dispatch_load_to(&ContainerId::Streams, &ActionLoad::Streams { type_name: "channel", id: "some_id" });
            future::ok(())
        })));
        let state = container_streams.get_state_owned();
    }
    */

    #[test]
    fn transport_manifests() {
        run(lazy(|| {
            let cinemeta_url = "https://v3-cinemeta.strem.io/manifest.json";
            let legacy_url = "https://opensubtitles.strem.io/stremioget/stremio/v1";
            let fut1 = AddonHTTPTransport::<Env>::from_url(&cinemeta_url)
                .manifest()
                .then(|res| {
                    if let Err(e) = res {
                        panic!("failed getting cinemeta manifest {:?}", e);
                    }
                    future::ok(())
                });
            let fut2 = AddonHTTPTransport::<Env>::from_url(&legacy_url)
                .manifest()
                .then(|res| {
                    if let Err(e) = res {
                        panic!("failed getting legacy manifest {:?}", e);
                    }
                    future::ok(())
                });
            fut1.join(fut2).map(|(_, _)| ())
        }));
    }

    #[test]
    fn get_videos() {
        run(lazy(|| {
            let transport_url = "https://v3-cinemeta.strem.io/manifest.json";
            AddonHTTPTransport::<Env>::from_url(&transport_url)
                .get(&ResourceRef::without_extra("meta", "series", "tt0386676"))
                .then(|res| {
                    match res {
                        Err(e) => panic!("failed getting metadata {:?}", e),
                        Ok(ResourceResponse::Meta { meta }) => {
                            //dbg!(&meta.videos);
                            assert!(meta.videos.len() > 0, "has videos")
                        }
                        _ => panic!("unexpected response"),
                    };
                    future::ok(())
                })
        }));
    }

    #[test]
    fn addon_collection() {
        run(lazy(|| {
            let collection_url = "https://api.strem.io/addonscollection.json";
            let req = Request::get(collection_url)
                .body(())
                .expect("builder cannot fail");
            Env::fetch_serde::<_, Vec<Descriptor>>(req).then(|res| {
                match res {
                    Err(e) => panic!("failed getting addon collection {:?}", e),
                    Ok(collection) => assert!(collection.len() > 0, "has addons"),
                };
                future::ok(())
            })
        }));
    }

    /*
    #[test]
    fn libitems() {
        use crate::libaddon::LibAddon;

        let auth_key = "".into();

        run(lazy(|| {
            let addon = LibAddon::<Env>::with_authkey(auth_key);
            addon.sync_with_api().then(|_stats| {
                //dbg!(&stats.unwrap());
                future::ok(())
            })
        }));
    }
    */

    #[test]
    fn sample_storage() {
        let key = "foo".to_owned();
        let value = "fooobar".to_owned();
        // Notihng in the beginning
        assert!(Env::get_storage::<String>(&key).wait().unwrap().is_none());
        // Then set and read
        // with RwLock and BTreeMap, set_storage takes 73993042ns for 10000 iterations (or 74ms)
        //  get_storage takes 42076632 (or 42ms) for 10000 iterations
        assert_eq!(Env::set_storage(&key, Some(&value)).wait().unwrap(), ());
        assert_eq!(
            Env::get_storage::<String>(&key).wait().unwrap(),
            Some(value)
        );
    }

    #[test]
    fn stremio_derive() {
        // Implement some dummy Ctx and contents
        struct Ctx {};
        impl Update for Ctx {
            fn update(&mut self, _: &Msg) -> Effects {
                dummy_effect()
            }
        }
        struct Content {};
        impl UpdateWithCtx<Ctx> for Content {
            fn update(&mut self, _: &Ctx, _: &Msg) -> Effects {
                dummy_effect()
            }
        }

        use stremio_derive::Model;
        #[derive(Model)]
        struct Model {
            pub ctx: Ctx,
            pub one: Content,
            pub two: Content,
        }
        let mut m = Model {
            ctx: Ctx {},
            one: Content {},
            two: Content {},
        };
        let fx = m.update(&Msg::Action(Action::LoadCtx));
        assert!(fx.has_changed, "has changed");
        assert_eq!(fx.effects.len(), 3, "proper number of effects");
    }
    fn dummy_effect() -> Effects {
        Effects::one(Box::new(future::ok(Msg::Action(Action::LoadCtx))))
    }

    // Testing the CatalogsGrouped model
    #[test]
    fn catalog_grouped() {
        use stremio_derive::Model;
        #[derive(Model)]
        struct Model {
            ctx: Ctx<Env>,
            catalogs: CatalogGrouped,
        }

        let mut app = Model {
            ctx: Default::default(),
            catalogs: Default::default(),
        };

        // @TODO use the macro
        dbg!(&app.catalogs);
        let msg = Msg::Action(Action::Load(ActionLoad::CatalogGrouped { extra: vec![] }));
        app.update(&msg);
        dbg!(&app.catalogs);
    }

    // Storage implementation
    // Uses reqwest (asynchronously) for fetch, and a BTreeMap storage
    use lazy_static::*;
    use std::collections::BTreeMap;
    use std::sync::RwLock;
    lazy_static! {
        static ref STORAGE: RwLock<BTreeMap<String, String>> = { Default::default() };
    }
    struct Env {}
    impl Environment for Env {
        fn fetch_serde<IN, OUT>(in_req: Request<IN>) -> EnvFuture<OUT>
        where
            IN: 'static + Serialize,
            OUT: 'static + DeserializeOwned,
        {
            let (parts, body) = in_req.into_parts();
            let method = reqwest::Method::from_bytes(parts.method.as_str().as_bytes())
                .expect("method is not valid for reqwest");
            let mut req = reqwest::r#async::Client::new().request(method, &parts.uri.to_string());
            // NOTE: both might be HeaderMap, so maybe there's a better way?
            for (k, v) in parts.headers.iter() {
                req = req.header(k.as_str(), v.as_ref());
            }
            // @TODO add content-type application/json
            // @TODO: if the response code is not 200, return an error related to that
            req = req.json(&body);
            let fut = req
                .send()
                .and_then(|mut res: reqwest::r#async::Response| res.json::<OUT>())
                .map_err(|e| e.into());
            Box::new(fut)
        }
        fn exec(fut: Box<dyn Future<Item = (), Error = ()>>) {
            spawn(fut);
        }
        fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<T>> {
            Box::new(future::ok(
                STORAGE
                    .read()
                    .unwrap()
                    .get(key)
                    .map(|v| serde_json::from_str(&v).unwrap()),
            ))
        }
        fn set_storage<T: 'static + Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()> {
            let mut storage = STORAGE.write().unwrap();
            match value {
                Some(v) => storage.insert(key.to_string(), serde_json::to_string(v).unwrap()),
                None => storage.remove(key),
            };
            Box::new(future::ok(()))
        }
    }
}
