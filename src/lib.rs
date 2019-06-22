pub mod addon_transport;
pub mod state_types;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::addon_transport::*;
    use crate::state_types::*;
    use crate::types::addons::{Descriptor, ResourceRef, ResourceRequest, ResourceResponse};
    use futures::future::lazy;
    use futures::{future, Future};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use tokio::executor::current_thread::spawn;
    use tokio::runtime::current_thread::run;

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
    // and the Runtime type
    #[test]
    fn catalog_grouped() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            catalogs: CatalogGrouped,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // Run a single dispatch of a Load msg
        let msg = Msg::Action(Action::Load(ActionLoad::CatalogGrouped { extra: vec![] }));
        run(runtime.dispatch(&msg));
        // since this is after the .run() has ended, this means all async effects
        // have processed
        {
            let state = &runtime.app.read().unwrap().catalogs;
            assert_eq!(state.groups.len(), 6, "groups is the right length");
            for g in state.groups.iter() {
                assert!(
                    match g.content {
                        Loadable::Ready(_) => true,
                        Loadable::Err(_) => true,
                        _ => false,
                    },
                    "group is Ready or Err"
                );
            }
        }

        // Now try the same, but with Search
        let extra = vec![("search".to_owned(), "grand tour".to_owned())];
        let msg = Msg::Action(Action::Load(ActionLoad::CatalogGrouped { extra }));
        run(runtime.dispatch(&msg));
        assert_eq!(
            runtime.app.read().unwrap().catalogs.groups.len(),
            4,
            "groups is the right length when searching"
        );
    }

    #[test]
    fn catalog_filtered() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            catalogs: CatalogFiltered,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        let req = ResourceRequest {
            base: "https://v3-cinemeta.strem.io/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "movie", "top"),
        };
        let action = Action::Load(ActionLoad::CatalogFiltered {
            resource_req: req.to_owned(),
        });
        run(runtime.dispatch(&action.into()));
        let state = &runtime.app.read().unwrap().catalogs;
        assert_eq!(state.selected, Some(req), "selected is right");
        assert_eq!(state.item_pages.len(), 1, "item_pages is the right length");
        match &state.item_pages[0] {
            Loadable::Ready(x) => assert_eq!(x.len(), 100, "right length of items"),
            _ => panic!("item_pages[0] is not Ready"),
        }
        //dbg!(serde_json::to_string(&state).unwrap());
    }

    #[test]
    fn streams() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            streams: Streams,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // If we login here with some dummy account, we can use this pretty nicely
        //let action = Action::UserOp(ActionUser::Login { email, password });
        //run(runtime.dispatch(&action.into()));

        // @TODO install some addons that provide streams
        let action = Action::Load(ActionLoad::Streams {
            type_name: "series".to_string(),
            id: "tt0773262:6:1".to_string(),
        });
        run(runtime.dispatch(&action.into()));
        let state = &runtime.app.read().unwrap().streams;
        assert_eq!(state.groups.len(), 2, "2 groups");
    }

    #[test]
    fn ctx_and_lib() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            lib_recent: LibRecent,
        }
        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // Log into a user, check if library synced correctly
        // We are always support to start with LoadCtx, even though it is not needed here
        run(runtime.dispatch(&Msg::Action(Action::LoadCtx)));

        // if this user gets deleted, the test will fail
        // @TODO register a new user instead
        let login_action = Action::UserOp(ActionUser::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into()
        });
        run(runtime.dispatch(&login_action.into()));
        // @TODO test if the addon collection is pulled
        let model = &runtime.app.read().unwrap();
        let first_content = model.ctx.content.clone();
        let first_lib = if let LibraryLoadable::Ready(l) = &model.ctx.library {
            assert!(l.items.len() > 0, "library has items");
            // LibRecent is "continue watching"
            assert!(model.lib_recent.recent.len() > 0, "has recent items");
            l.clone()
        } else {
            panic!("library must be Ready")
        };

        // New runtime, just LoadCtx, to see if the ctx was persisted
        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);
        assert_eq!(runtime.app.read().unwrap().ctx.is_loaded, false);
        run(runtime.dispatch(&Msg::Action(Action::LoadCtx)));
        {
            let ctx = &runtime.app.read().unwrap().ctx;
            assert_eq!(&first_content, &ctx.content, "content is the same");
            if let LibraryLoadable::Ready(l) = &ctx.library {
                assert_eq!(&first_lib, l, "loaded lib is same as synced");
            } else {
                panic!("library must be Ready")
            }
            assert_eq!(ctx.is_loaded, true);
        }

        // Logout and expect everything to be reset
        let logout_action = Action::UserOp(ActionUser::Logout);
        run(runtime.dispatch(&logout_action.into()));
        {
            let ctx = &runtime.app.read().unwrap().ctx;
            assert!(ctx.content.auth.is_none(), "logged out");
            assert!(ctx.content.addons.len() > 0, "has addons");
            if let LibraryLoadable::Ready(l) = &ctx.library {
                assert_eq!(l.items.len(), 0, "library must be empty");
            } else {
                panic!("library must be Ready")
            }
        }
        // @TODO we will try to insert an item and see if it will be persisted
        // currently, LibUpdate does not support this
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
        fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()> {
            let mut storage = STORAGE.write().unwrap();
            match value {
                Some(v) => storage.insert(key.to_string(), serde_json::to_string(v).unwrap()),
                None => storage.remove(key),
            };
            Box::new(future::ok(()))
        }
    }
}
