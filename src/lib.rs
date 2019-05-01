pub mod addon_transport;
pub mod middlewares;
pub mod state_types;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::addon_transport::*;
    use crate::middlewares::*;
    use crate::state_types::*;
    use crate::types::addons::{ResourceRef, ResourceRequest};
    use enclose::*;
    use futures::future::lazy;
    use futures::{future, Future};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::rc::Rc;
    use tokio::executor::current_thread::spawn;
    use tokio::runtime::current_thread::run;

    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
    enum ContainerId {
        Board,
        Discover,
    }

    #[test]
    fn middlewares() {
        // to make sure we can't use 'static
        inner_middlewares();
    }
    fn inner_middlewares() {
        // @TODO: Fix: the assumptions we are testing against are pretty much based on the current
        // official addons; e.g. assuming 6 groups, or 4 groups when searching
        // @TODO test what happens with no handlers
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
            transport_url: "https://v3-cinemeta.strem.io/manifest.json".to_owned(),
            resource_ref: ResourceRef::without_extra("catalog", "movie", "top"),
        };
        run(lazy(enclose!((muxer, resource_req) move || {
            muxer.dispatch_load_to(&ContainerId::Discover, &ActionLoad::CatalogFiltered { resource_req });
            future::ok(())
        })));
        let state = container_filtered.get_state_owned();
        assert_eq!(state.selected, Some(resource_req), "selected is right");
        assert_eq!(state.item_pages.len(), 1, "item_pages is the right length");
        assert!(state.item_pages[0].is_ready(), "first page is ready");

        /*
        // @TODO
        run(lazy(enclose!((muxer, resource_req) move || {
            muxer.dispatch_load_to(&ContainerId::Streams, &ActionLoad::Streams { type_name: "channel", id: "some_id" });
            future::ok(())
        })));
        let state = container_streams.get_state_owned();
        */
    }

    #[test]
    fn transport_manifests() {
        run(lazy(|| {
            let cinemeta_url = "https://v3-cinemeta.strem.io/manifest.json";
            let legacy_url = "https://opensubtitles.strem.io/stremioget/stremio/v1";
            let fut1 = AddonHTTPTransport::<Env>::manifest(cinemeta_url).then(|res| {
                if let Err(e) = res {
                    panic!("failed getting cinemeta manifest {:?}", e);
                }
                future::ok(())
            });
            let fut2 = AddonHTTPTransport::<Env>::manifest(legacy_url).then(|res| {
                if let Err(e) = res {
                    panic!("failed getting legacy manifest {:?}", e);
                }
                future::ok(())
            });
            fut1.join(fut2).map(|(_, _)| ())
        }));
    }

    #[test]
    fn sample_storage() {
        let key = "foo".to_owned();
        let value = "fooobar".to_owned();
        // Nothing in the beginning
        assert!(Env::get_storage::<String>(&key).wait().unwrap().is_none());
        // Then set and read
        assert_eq!(Env::set_storage(&key, Some(&value)).wait().unwrap(), ());
        assert_eq!(
            Env::get_storage::<String>(&key).wait().unwrap(),
            Some(Box::new(value))
        );
        // performance with sled
        // 10,000 iterations
        // set_storage: 2066168715ns (~2s)
        // get_storage: 225117363ns
    }

    use lazy_static::*;
    use sled::Db;
    lazy_static! {
        static ref STORAGE: sled::Db = {
            Db::start_default("./store").expect("failed to start sled")
        };
    }
    struct Env {}
    impl Environment for Env {
        fn fetch_serde<IN, OUT>(in_req: Request<IN>) -> EnvFuture<Box<OUT>>
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
                .map(|res| Box::new(res))
                .map_err(|e| e.into());
            Box::new(fut)
        }
        fn exec(fut: Box<Future<Item = (), Error = ()>>) {
            spawn(fut);
        }
        fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<Box<T>>> {
            let opt = match STORAGE.get(key.as_bytes()) {
                Ok(s) => s,
                Err(e) => return Box::new(future::err(e.into())),
            };
            Box::new(future::ok(
                opt.map(|v| Box::new(serde_json::from_slice(&*v).unwrap())),
            ))
        }
        fn set_storage<T: 'static + Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()> {
            let res = match value {
                Some(v) => STORAGE.set(key.as_bytes(), serde_json::to_string(v).unwrap().as_bytes()),
                None => STORAGE.del(key),
            };
            match res {
                Ok(_) => Box::new(future::ok(())),
                Err(e) => Box::new(future::err(e.into())),
            }
        }
    }
}
