pub mod addon_transport;
pub mod middlewares;
pub mod state_types;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::addon_transport::*;
    use crate::middlewares::*;
    use crate::state_types::*;
    use enclose::*;
    use futures::future::lazy;
    use futures::{future, Future};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::rc::Rc;
    use tokio::executor::current_thread::spawn;
    use tokio::runtime::current_thread::run;
    
    #[derive(Debug, Clone)]
    enum ContainerId {
        Board,
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
        let muxer = Rc::new(ContainerMuxer::new(
            vec![
                Box::new(ContextMiddleware::<Env>::new()),
                Box::new(AddonsMiddleware::<Env>::new()),
            ],
            vec![(
                ContainerId::Board,
                container.clone() as Rc<dyn ContainerInterface>,
            )],
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
        fn get_storage<T: 'static + DeserializeOwned>(_key: &str) -> EnvFuture<Option<Box<T>>> {
            Box::new(future::ok(None))
        }
        fn set_storage<T: 'static + Serialize>(_key: &str, _value: Option<&T>) -> EnvFuture<()> {
            Box::new(future::err("unimplemented".into()))
        }
    }
}
