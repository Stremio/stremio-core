#![allow(clippy::module_inception)]

pub mod addon_transport;
pub mod constants;
pub mod state_types;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::addon_transport::*;
    use crate::state_types::models::addon_details::AddonDetails;
    use crate::state_types::models::catalog_with_filters::CatalogWithFilters;
    use crate::state_types::models::catalogs_with_extra::CatalogsWithExtra;
    use crate::state_types::models::common::*;
    use crate::state_types::models::continue_watching_preview::ContinueWatchingPreview;
    use crate::state_types::models::ctx::*;
    use crate::state_types::models::library_with_filters::{LibraryWithFilters, NotRemovedFilter};
    use crate::state_types::models::meta_details::MetaDetails;
    use crate::state_types::models::notifications::Notifications;
    use crate::state_types::models::player::Player;
    use crate::state_types::models::streaming_server::StreamingServer;
    use crate::state_types::msg::*;
    use crate::state_types::*;
    use crate::types::addons::*;
    use crate::types::api::AuthRequest;
    use crate::types::*;
    use chrono::{DateTime, Utc};
    use futures::future::lazy;
    use futures::{future, Future};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::cmp::Ordering;
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
            let transport_url = "http://127.0.0.1:7001/manifest.json";
            AddonHTTPTransport::<Env>::from_url(&transport_url)
                .get(&ResourceRef::without_extra("meta", "series", "st2"))
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

    #[test]
    fn addon_details() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            addon_details: AddonDetails,
            catalogs: CatalogWithFilters<MetaPreview>,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        assert!(
            addon_desc.manifest.id == "com.stremio.taddon",
            "id is correct"
        );
        assert!(
            addon_desc.transport_url == "http://127.0.0.1:7001/manifest.json",
            "transport url is correct"
        );

        // testing the unload action
        let unload = Msg::Action(Action::Unload);
        run(runtime.dispatch(&unload));
        assert!(
            match runtime.app.write().unwrap().addon_details.selected {
                None => true,
                _ => false,
            },
            "selected is None"
        );
        assert!(
            match runtime.app.write().unwrap().addon_details.addon {
                None => true,
                _ => false,
            },
            "addon is None"
        );

        // testing with incorrect url
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://example.com/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Err(_) => assert!(true, "addon errored"),
            _ => panic!("the addon didn't throw error"),
        };
    }

    #[test]
    fn install_and_uninstall_addon() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            addon_details: AddonDetails,
        }

        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        let addons_len_before_install = runtime.app.read().unwrap().ctx.profile.addons.len();
        let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
        run(runtime.dispatch(&install_action));

        let addons = &runtime.app.read().unwrap().ctx.profile.addons.to_owned();
        let addons_len_after_install = addons.len();
        let test_addon = addons
            .iter()
            .find(|c| c.transport_url == "http://127.0.0.1:7001/manifest.json")
            .expect("could not find test addon");
        assert_eq!(
            addons_len_after_install - addons_len_before_install,
            1,
            "test addon is installed"
        );
        assert_eq!(
            &test_addon.transport_url, "http://127.0.0.1:7001/manifest.json",
            "correct test addon is installed"
        );
        let uninstall_action = Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(
            test_addon.transport_url.to_owned(),
        )));
        run(runtime.dispatch(&uninstall_action));
        let addons_len_after_uninstall = runtime.app.read().unwrap().ctx.profile.addons.len();
        assert_eq!(
            addons_len_before_install, addons_len_after_uninstall,
            "test addon is uninstalled"
        );
    }

    #[test]
    fn login_logout() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            lib_recent: ContinueWatchingPreview,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        assert!(
            match runtime.app.write().unwrap().ctx.profile.auth {
                None => true,
                _ => false,
            },
            "there is no user"
        );

        // Log into a user, check if library synced correctly
        run(runtime.dispatch(&Msg::Action(Action::Load(ActionLoad::Ctx))));

        // if this user gets deleted, the test will fail
        // @TODO register a new user instead
        let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })));
        run(runtime.dispatch(&login_msg));
        assert!(
            match &runtime.app.write().unwrap().ctx.profile.auth {
                Some(auth) => {
                    if auth.user.email == "ctxandlib@stremio.com" {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            "user logged in successfully"
        );

        // Logout and expect everything to be reset
        let logout_action = Msg::Action(Action::Ctx(ActionCtx::Logout));
        run(runtime.dispatch(&logout_action));
        assert!(
            match runtime.app.write().unwrap().ctx.profile.auth {
                None => true,
                _ => false,
            },
            "user logged out successfully"
        );
    }

    #[test]
    fn update_profile_settings() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })));
        run(runtime.dispatch(&login_msg));

        // changing settings with proper values
        let settings = profile::Settings {
            subtitles_language: "bg".to_string(),
            subtitles_size: 150,
            ..runtime.app.read().unwrap().ctx.profile.settings.to_owned()
        };
        let update_action =
            Msg::Action(Action::Ctx(ActionCtx::UpdateSettings(settings.to_owned())));
        run(runtime.dispatch(&update_action));
        assert_eq!(
            settings.to_owned(),
            runtime.app.read().unwrap().ctx.profile.settings,
            "settings are updated correctly"
        );
    }

    #[test]
    fn add_to_remove_from_library() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            catalogs: CatalogWithFilters<MetaPreview>,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        // Log into a user, check if library synced correctly
        run(runtime.dispatch(&Msg::Action(Action::Load(ActionLoad::Ctx))));

        // if this user gets deleted, the test will fail
        // @TODO register a new user instead
        let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })));
        run(runtime.dispatch(&login_msg));
        let req = ResourceRequest {
            base: "http://127.0.0.1:7001/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "movie", "test"),
        };
        let action = Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(
            models::catalog_with_filters::Selected {
                request: req.to_owned(),
            },
        )));
        run(runtime.dispatch_with(|model| model.catalogs.update(&model.ctx, &action)));
        let content = match runtime
            .app
            .write()
            .unwrap()
            .catalogs
            .to_owned()
            .catalog_resource
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("content not ready, but instead: {:?}", x),
        };
        let first_meta_preview = &content[0];
        let items = &runtime.app.read().unwrap().ctx.library.items.to_owned();
        let has_item = match items.get(&first_meta_preview.id) {
            Some(item) => {
                if item.removed {
                    false
                } else {
                    true
                }
            }
            None => false,
        };

        if has_item {
            let remove_action = Msg::Action(Action::Ctx(ActionCtx::RemoveFromLibrary(
                first_meta_preview.id.to_owned(),
            )));
            run(runtime.dispatch(&remove_action));
            assert!(
                &runtime
                    .app
                    .read()
                    .unwrap()
                    .ctx
                    .library
                    .items
                    .to_owned()
                    .get(&first_meta_preview.id)
                    .unwrap()
                    .removed,
                "item is removed"
            );
            let add_action = Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
                first_meta_preview.to_owned(),
            )));
            run(runtime.dispatch(&add_action));
            assert!(
                !&runtime
                    .app
                    .read()
                    .unwrap()
                    .ctx
                    .library
                    .items
                    .to_owned()
                    .get(&first_meta_preview.id)
                    .unwrap()
                    .removed,
                "item is added"
            );
        } else {
            let add_action = Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
                first_meta_preview.to_owned(),
            )));
            run(runtime.dispatch(&add_action));
            assert!(
                !&runtime
                    .app
                    .read()
                    .unwrap()
                    .ctx
                    .library
                    .items
                    .to_owned()
                    .get(&first_meta_preview.id)
                    .unwrap()
                    .removed,
                "item is added"
            );
            let remove_action = Msg::Action(Action::Ctx(ActionCtx::RemoveFromLibrary(
                first_meta_preview.id.to_owned(),
            )));
            run(runtime.dispatch(&remove_action));
            assert!(
                &runtime
                    .app
                    .read()
                    .unwrap()
                    .ctx
                    .library
                    .items
                    .to_owned()
                    .get(&first_meta_preview.id)
                    .unwrap()
                    .removed,
                "item is removed"
            );
        };
    }

    #[test]
    fn rewind_library_item() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            addon_details: AddonDetails,
            catalogs: CatalogWithFilters<MetaPreview>,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        // Testing without user
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
        run(runtime.dispatch(&install_action));
        let req = ResourceRequest {
            base: "http://127.0.0.1:7001/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "movie", "test"),
        };
        let action = Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(
            models::catalog_with_filters::Selected {
                request: req.to_owned(),
            },
        )));
        run(runtime.dispatch_with(|model| model.catalogs.update(&model.ctx, &action)));
        let content = match runtime
            .app
            .write()
            .unwrap()
            .catalogs
            .to_owned()
            .catalog_resource
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("content not ready, but instead: {:?}", x),
        };
        let first_meta_preview = &content[0];
        let add_action = Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
            first_meta_preview.to_owned(),
        )));
        run(runtime.dispatch(&add_action));

        let mut lib_item = runtime
            .app
            .read()
            .unwrap()
            .ctx
            .library
            .items
            .get(&first_meta_preview.id)
            .unwrap()
            .to_owned();

        // set time_offset to number greater than 0
        lib_item.state.time_offset = 10;
        run(runtime.dispatch(&Msg::Internal(Internal::UpdateLibraryItem(
            lib_item.to_owned(),
        ))));
        assert!(
            runtime
                .app
                .read()
                .unwrap()
                .ctx
                .library
                .items
                .get(&first_meta_preview.id)
                .unwrap()
                .to_owned()
                .state
                .time_offset
                == 10,
            "time offset is right"
        );
        let rewind_action = Msg::Action(Action::Ctx(ActionCtx::RewindLibraryItem(lib_item.id)));
        run(runtime.dispatch(&rewind_action));
        assert!(
            runtime
                .app
                .read()
                .unwrap()
                .ctx
                .library
                .items
                .get(&first_meta_preview.id)
                .unwrap()
                .to_owned()
                .state
                .time_offset
                == 0,
            "time offset is rewinded"
        );

        // Testing with user

        // Log into a user, check if library synced correctly
        run(runtime.dispatch(&Msg::Action(Action::Load(ActionLoad::Ctx))));

        // if this user gets deleted, the test will fail
        // @TODO register a new user instead
        let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })));
        run(runtime.dispatch(&login_msg));
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
        run(runtime.dispatch(&install_action));
        let req = ResourceRequest {
            base: "http://127.0.0.1:7001/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "movie", "test"),
        };
        let action = Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(
            models::catalog_with_filters::Selected {
                request: req.to_owned(),
            },
        )));
        run(runtime.dispatch_with(|model| model.catalogs.update(&model.ctx, &action)));
        let content = match runtime
            .app
            .write()
            .unwrap()
            .catalogs
            .to_owned()
            .catalog_resource
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("content not ready, but instead: {:?}", x),
        };
        let first_meta_preview = &content[0];
        let add_action = Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
            first_meta_preview.to_owned(),
        )));
        run(runtime.dispatch(&add_action));

        let mut lib_item = runtime
            .app
            .read()
            .unwrap()
            .ctx
            .library
            .items
            .get(&first_meta_preview.id)
            .unwrap()
            .to_owned();

        // set time_offset to number greater than 0
        lib_item.state.time_offset = 10;
        run(runtime.dispatch(&Msg::Internal(Internal::UpdateLibraryItem(
            lib_item.to_owned(),
        ))));
        assert!(
            runtime
                .app
                .read()
                .unwrap()
                .ctx
                .library
                .items
                .get(&first_meta_preview.id)
                .unwrap()
                .to_owned()
                .state
                .time_offset
                == 10,
            "time offset is right"
        );
        let rewind_action = Msg::Action(Action::Ctx(ActionCtx::RewindLibraryItem(lib_item.id)));
        run(runtime.dispatch(&rewind_action));
        assert!(
            runtime
                .app
                .read()
                .unwrap()
                .ctx
                .library
                .items
                .get(&first_meta_preview.id)
                .unwrap()
                .to_owned()
                .state
                .time_offset
                == 0,
            "time offset is rewinded"
        );
    }

    // streaming server must be available
    #[test]
    fn action_streaming_server() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            streaming_server: StreamingServer,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        let reload = Msg::Action(Action::StreamingServer(ActionStreamingServer::Reload));
        run(runtime.dispatch(&reload));
        assert!(
            match &runtime.app.write().unwrap().streaming_server.settings {
                Some(_s) => true,
                _ => false,
            },
            "settings are loaded"
        );
        let ready_settings = match runtime
            .app
            .read()
            .unwrap()
            .streaming_server
            .settings
            .to_owned()
            .unwrap()
        {
            Loadable::Ready(x) => x,
            x => panic!("settings not ready, but instead: {:?}", x),
        };

        // changing settings with proper values
        let settings = models::streaming_server::Settings {
            bt_max_connections: 25,
            ..ready_settings.to_owned()
        };
        let update_action = Msg::Action(Action::StreamingServer(
            ActionStreamingServer::UpdateSettings(settings.to_owned()),
        ));
        run(runtime.dispatch(&update_action));

        // changing settings with proper values
        let settings = models::streaming_server::Settings {
            bt_max_connections: 45,
            ..ready_settings.to_owned()
        };
        let update_action = Msg::Action(Action::StreamingServer(
            ActionStreamingServer::UpdateSettings(settings.to_owned()),
        ));
        run(runtime.dispatch(&update_action));
        let ready_settings = Loadable::Ready(settings);
        assert_eq!(
            runtime.app.read().unwrap().streaming_server.settings,
            Some(ready_settings),
            "settings are updated correctly"
        );
    }

    #[test]
    fn action_player() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            addon_details: AddonDetails,
            meta_details: MetaDetails,
            player: Player,
            lib_recent: ContinueWatchingPreview,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // if this user gets deleted, the test will fail
        // @TODO register a new user instead
        let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })));
        run(runtime.dispatch(&login_msg));

        // install addon that provides streams
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
        run(runtime.dispatch(&install_action));

        let action = Msg::Action(Action::Load(ActionLoad::MetaDetails(
            models::meta_details::Selected {
                meta_resource_ref: ResourceRef {
                    resource: "meta".to_string(),
                    type_name: "series".to_string(),
                    id: "st2".to_string(),
                    extra: vec![],
                },
                streams_resource_ref: Some(ResourceRef {
                    resource: "stream".to_string(),
                    type_name: "series".to_string(),
                    id: "st2v1".to_string(),
                    extra: vec![],
                }),
            },
        )));
        run(runtime.dispatch(&action));

        let first_meta_resource_content =
            match runtime.app.write().unwrap().meta_details.meta_resources[0]
                .to_owned()
                .content
            {
                Loadable::Ready(x) => x,
                x => panic!("content not ready, but instead: {:?}", x),
            };
        let stream = &first_meta_resource_content.videos[0].streams[0];
        let req = runtime.app.write().unwrap().meta_details.meta_resources[0]
            .request
            .to_owned();
        let video_id = &first_meta_resource_content.videos[0].id;
        let player = Msg::Action(Action::Load(ActionLoad::Player(models::player::Selected {
            stream: stream.to_owned(),
            stream_resource_request: Default::default(),
            meta_resource_request: Some(req),
            subtitles_resource_ref: Default::default(),
            video_id: Some(video_id.to_owned()),
        })));
        run(runtime.dispatch(&player));

        // testing the UpdateLibraryItemState action
        let player_lib_item = runtime
            .app
            .read()
            .unwrap()
            .player
            .lib_item
            .to_owned()
            .unwrap();
        let rewind_action = Msg::Action(Action::Ctx(ActionCtx::RewindLibraryItem(
            player_lib_item.id.to_owned(),
        )));
        run(runtime.dispatch(&rewind_action));

        // set time and duration with proper numbers
        let update_action = Msg::Action(Action::Player(ActionPlayer::UpdateLibraryItemState {
            time: 10,
            duration: 100,
        }));
        run(runtime.dispatch(&update_action));
        assert_eq!(
            runtime
                .app
                .read()
                .unwrap()
                .player
                .lib_item
                .to_owned()
                .unwrap()
                .state
                .time_offset,
            10 as u64,
            "time offset is correct"
        );
        assert_eq!(
            runtime
                .app
                .read()
                .unwrap()
                .player
                .lib_item
                .to_owned()
                .unwrap()
                .state
                .duration,
            100 as u64,
            "duration is correct"
        );

        // testing the PushToLibrary action
        let push_action = Msg::Action(Action::Player(ActionPlayer::PushToLibrary));
        run(runtime.dispatch(&push_action));
        let lib_items = &runtime.app.read().unwrap().lib_recent.lib_items;
        let lib_item = lib_items
            .iter()
            .find(|i| i.id == player_lib_item.id.to_owned())
            .expect("could not find lib item");
        assert_eq!(
            lib_item.state.time_offset, 10 as u64,
            "time offset is correct"
        );
        assert_eq!(lib_item.state.duration, 100 as u64, "duration is correct");
    }

    #[test]
    fn push_addons_to_api() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            addon_details: AddonDetails,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        // Log into a user
        let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })));
        run(runtime.dispatch(&login_msg));
        let addons = &runtime.app.read().unwrap().ctx.profile.addons.to_owned();

        // check if user addons contain the test addon
        let has_addon = addons
            .iter()
            .any(|addon| addon.transport_url == "http://127.0.0.1:7001/manifest.json");

        if has_addon {
            let addons_len_before_uninstall = &runtime.app.read().unwrap().ctx.profile.addons.len();
            let test_addon = addons
                .iter()
                .find(|c| c.transport_url == "http://127.0.0.1:7001/manifest.json")
                .expect("could not find test addon");
            let uninstall_action = Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(
                test_addon.transport_url.to_owned(),
            )));
            run(runtime.dispatch(&uninstall_action));
            let action = Msg::Action(Action::Ctx(ActionCtx::PushAddonsToAPI));
            run(runtime.dispatch(&action));

            let logout_action = Msg::Action(Action::Ctx(ActionCtx::Logout));
            run(runtime.dispatch(&logout_action));
            let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
                email: "ctxandlib@stremio.com".into(),
                password: "ctxandlib".into(),
            })));
            run(runtime.dispatch(&login_msg));
            let addons_len_after_relogin = &runtime.app.read().unwrap().ctx.profile.addons.len();
            assert_eq!(
                addons_len_before_uninstall - addons_len_after_relogin,
                1,
                "addons are pushed to API"
            );
        } else {
            let addons_len_before_install = &runtime.app.read().unwrap().ctx.profile.addons.len();
            let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
                models::addon_details::Selected {
                    transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
                },
            )));
            run(runtime.dispatch(&addon_details));
            let addon_desc = match runtime
                .app
                .write()
                .unwrap()
                .addon_details
                .addon
                .to_owned()
                .unwrap()
                .content
            {
                Loadable::Ready(x) => x,
                x => panic!("addon not ready, but instead: {:?}", x),
            };
            let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
            run(runtime.dispatch(&install_action));
            let action = Msg::Action(Action::Ctx(ActionCtx::PushAddonsToAPI));
            run(runtime.dispatch(&action));

            let logout_action = Msg::Action(Action::Ctx(ActionCtx::Logout));
            run(runtime.dispatch(&logout_action));
            let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
                email: "ctxandlib@stremio.com".into(),
                password: "ctxandlib".into(),
            })));
            run(runtime.dispatch(&login_msg));
            let addons_len_after_relogin = &runtime.app.read().unwrap().ctx.profile.addons.len();
            assert_eq!(
                addons_len_after_relogin - addons_len_before_install,
                1,
                "addons are pushed to API"
            );
        }
    }

    #[test]
    fn library_with_filters() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            library: LibraryWithFilters<NotRemovedFilter>,
            catalogs: CatalogWithFilters<MetaPreview>,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        // Log into a user, check if library synced correctly
        run(runtime.dispatch(&Msg::Action(Action::Load(ActionLoad::Ctx))));

        // if this user gets deleted, the test will fail
        // @TODO register a new user instead
        let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })));
        run(runtime.dispatch(&login_msg));

        let req = ResourceRequest {
            base: "http://127.0.0.1:7001/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "movie", "test"),
        };
        let action = Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(
            models::catalog_with_filters::Selected {
                request: req.to_owned(),
            },
        )));
        run(runtime.dispatch_with(|model| model.catalogs.update(&model.ctx, &action)));
        let content = match runtime
            .app
            .write()
            .unwrap()
            .catalogs
            .to_owned()
            .catalog_resource
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("content not ready, but instead: {:?}", x),
        };
        let first_meta_preview = &content[0];
        let add_action = Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
            first_meta_preview.to_owned(),
        )));
        run(runtime.dispatch(&add_action));

        let req = ResourceRequest {
            base: "http://127.0.0.1:7001/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "series", "test"),
        };
        let action = Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(
            models::catalog_with_filters::Selected {
                request: req.to_owned(),
            },
        )));
        run(runtime.dispatch_with(|model| model.catalogs.update(&model.ctx, &action)));
        let content = match runtime
            .app
            .write()
            .unwrap()
            .catalogs
            .to_owned()
            .catalog_resource
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("content not ready, but instead: {:?}", x),
        };
        let first_meta_preview = &content[0];
        let second_meta_preview = &content[1];
        let add_action = Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
            first_meta_preview.to_owned(),
        )));
        run(runtime.dispatch(&add_action));
        let add_action = Msg::Action(Action::Ctx(ActionCtx::AddToLibrary(
            second_meta_preview.to_owned(),
        )));
        run(runtime.dispatch(&add_action));

        let lib_action = Msg::Action(Action::Load(ActionLoad::LibraryWithFilters(
            models::library_with_filters::Selected {
                type_name: Some("movie".into()),
                sort: models::library_with_filters::Sort::LastWatched,
            },
        )));
        run(runtime.dispatch(&lib_action));
        assert_eq!(
            &runtime.app.read().unwrap().library.lib_items[0].type_name,
            "movie",
            "first item has type movie"
        );

        let lib_action = Msg::Action(Action::Load(ActionLoad::LibraryWithFilters(
            models::library_with_filters::Selected {
                type_name: Some("series".into()),
                sort: models::library_with_filters::Sort::Name,
            },
        )));
        run(runtime.dispatch(&lib_action));
        assert_eq!(
            &runtime.app.read().unwrap().library.lib_items[0].type_name,
            "series",
            "first item has type series"
        );
        let name_comparing = runtime.app.read().unwrap().library.lib_items[0]
            .name
            .cmp(&runtime.app.read().unwrap().library.lib_items[1].name);
        assert_eq!(
            Ordering::Less,
            name_comparing,
            "items are arranged alphabetically"
        );

        // testing the unload action
        let unload = Msg::Action(Action::Unload);
        run(runtime.dispatch(&unload));
        assert!(
            match runtime.app.write().unwrap().library.selected {
                None => true,
                _ => false,
            },
            "selected is None"
        );
        assert_eq!(
            runtime.app.read().unwrap().library.lib_items.len(),
            0,
            "lib items are empty"
        );
    }

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
        let fx = m.update(&Msg::Action(Action::Load(ActionLoad::Ctx)));
        assert!(fx.has_changed, "has changed");
        assert_eq!(fx.effects.len(), 3, "proper number of effects");
    }
    fn dummy_effect() -> Effects {
        Effects::one(Box::new(future::ok(Msg::Action(Action::Load(
            ActionLoad::Ctx,
        )))))
    }

    // Testing the CatalogsWithExtra model
    // and the Runtime type
    #[test]
    fn catalog_with_extra() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            catalogs: CatalogsWithExtra,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // Run a single dispatch of a Load msg
        let msg = Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra(
            models::catalogs_with_extra::Selected { extra: vec![] },
        )));
        run(runtime.dispatch(&msg));
        // since this is after the .run() has ended, this means all async effects
        // have processed
        {
            let state = &runtime.app.read().unwrap().catalogs;
            assert_eq!(
                state.catalog_resources.len(),
                7,
                "groups is the right length"
            );
            for g in state.catalog_resources.iter() {
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
        let msg = Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra(
            models::catalogs_with_extra::Selected { extra },
        )));
        run(runtime.dispatch(&msg));
        assert_eq!(
            runtime.app.read().unwrap().catalogs.catalog_resources.len(),
            5,
            "groups is the right length when searching"
        );

        // testing the unload action
        let unload = Msg::Action(Action::Unload);
        run(runtime.dispatch(&unload));
        assert!(
            match runtime.app.write().unwrap().catalogs.selected {
                None => true,
                _ => false,
            },
            "selected is None"
        );
        assert_eq!(
            runtime.app.read().unwrap().catalogs.catalog_resources.len(),
            0,
            "catalog resources are empty"
        );
    }

    #[test]
    fn catalog_filtered() {
        use stremio_derive::Model;
        #[derive(Model, Debug)]
        struct Model {
            ctx: Ctx<Env>,
            addon_details: AddonDetails,
            catalogs: CatalogWithFilters<MetaPreview>,
        }

        let app = Model {
            ctx: Default::default(),
            addon_details: Default::default(),
            catalogs: CatalogWithFilters {
                selected: Default::default(),
                selectable: Default::default(),
                catalog_resource: Default::default(),
            },
        };
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
        run(runtime.dispatch(&install_action));
        let state = runtime.app.read().unwrap().catalogs.to_owned();
        let test_catalog = state
            .selectable
            .catalogs
            .iter()
            .find(|c| {
                c.request.base == "http://127.0.0.1:7001/manifest.json"
                    && c.request.path.type_name == "movie"
            })
            .expect("could not find test catalog");

        let req = ResourceRequest {
            base: "http://127.0.0.1:7001/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "movie", "test"),
        };
        let action = Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(
            models::catalog_with_filters::Selected {
                request: req.to_owned(),
            },
        )));
        run(runtime.dispatch_with(|model| model.catalogs.update(&model.ctx, &action)));
        // Clone the state so that we don't keep a lock on .app
        let state = runtime.app.read().unwrap().catalogs.to_owned();
        assert!(state.catalog_resource.is_some(), "selected is right");
        assert!(
            state
                .catalog_resource
                .as_ref()
                .unwrap()
                .request
                .path
                .type_name
                .eq(&state.selectable.types[0].request.path.type_name),
            "first type is selected"
        );
        assert!(
            state
                .catalog_resource
                .as_ref()
                .unwrap()
                .request
                .path
                .type_name
                .eq(&test_catalog.request.path.type_name),
            "first catalog is selected"
        );
        assert_eq!(
            state.selectable.types[0].request.path.type_name, "movie",
            "first type is movie"
        );
        assert!(state.selectable.catalogs.len() > 0, "has catalogs");
        match &state.catalog_resource {
            Some(ResourceLoadable {
                content: Loadable::Ready(x),
                ..
            }) => assert_eq!(x.len(), 100, "right length of items"),
            x => panic!("item_pages[0] is not Ready, but instead: {:?}", x),
        };

        // Verify that pagination works

        assert!(
            state.selectable.has_next_page,
            "there should be a next page"
        );
        let load_next = ResourceRequest {
            base: "http://127.0.0.1:7001/manifest.json".to_owned(),
            path: ResourceRef::with_extra(
                "catalog",
                "movie",
                "test",
                &[("skip".to_owned(), "100".to_owned())],
            ),
        };
        let action = Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(
            models::catalog_with_filters::Selected { request: load_next },
        )));
        run(runtime.dispatch(&action));
        let state = runtime.app.read().unwrap().catalogs.to_owned();
        assert!(
            state
                .catalog_resource
                .as_ref()
                .unwrap()
                .request
                .path
                .type_name
                .eq(&state.selectable.types[0].request.path.type_name),
            "first type is still selected"
        );
        assert!(
            state
                .catalog_resource
                .as_ref()
                .unwrap()
                .request
                .eq_no_extra(&test_catalog.request),
            "first catalog is still selected"
        );
        assert_eq!(
            state
                .catalog_resource
                .as_ref()
                .expect("there must be .catalog_resource")
                .request
                .path
                .get_extra_first_val("skip"),
            Some("100"),
            "skip extra is correct"
        );
        assert!(
            state.selectable.has_next_page,
            "there should be a next page"
        );
        assert!(
            state.selectable.has_prev_page,
            "there should be a prev page"
        );

        // testing the unload action
        let unload = Msg::Action(Action::Unload);
        run(runtime.dispatch(&unload));
        assert!(
            match runtime.app.write().unwrap().catalogs.selected {
                None => true,
                _ => false,
            },
            "selected is None"
        );
        assert!(
            match runtime.app.write().unwrap().catalogs.catalog_resource {
                None => true,
                _ => false,
            },
            "catalog resource is None"
        );
    }

    #[test]
    fn meta_details() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            meta_details: MetaDetails,
            addon_details: AddonDetails,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // install addon that provides streams
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
        run(runtime.dispatch(&install_action));

        let action = Msg::Action(Action::Load(ActionLoad::MetaDetails(
            models::meta_details::Selected {
                meta_resource_ref: ResourceRef {
                    resource: "meta".to_string(),
                    type_name: "series".to_string(),
                    id: "st2".to_string(),
                    extra: vec![],
                },
                streams_resource_ref: Some(ResourceRef {
                    resource: "stream".to_string(),
                    type_name: "series".to_string(),
                    id: "st2v1".to_string(),
                    extra: vec![],
                }),
            },
        )));
        run(runtime.dispatch(&action));
        match &runtime.app.write().unwrap().meta_details.meta_resources[0].content {
            Loadable::Ready(first_meta_resource) => {
                assert_eq!(first_meta_resource.id, "st2", "id is the same")
            }
            x => panic!("content not ready, but instead: {:?}", x),
        };

        // testing the unload action
        let unload = Msg::Action(Action::Unload);
        run(runtime.dispatch(&unload));
        assert!(
            match runtime.app.write().unwrap().meta_details.selected {
                None => true,
                _ => false,
            },
            "selected is None"
        );
        assert_eq!(
            runtime
                .app
                .read()
                .unwrap()
                .meta_details
                .meta_resources
                .len(),
            0,
            "meta resources are empty"
        );
        assert_eq!(
            runtime
                .app
                .read()
                .unwrap()
                .meta_details
                .streams_resources
                .len(),
            0,
            "streams resources are empty"
        );
    }

    #[test]
    fn streams() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            meta_details: MetaDetails,
            addon_details: AddonDetails,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // install addon that provides streams
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
        run(runtime.dispatch(&install_action));

        let action = Msg::Action(Action::Load(ActionLoad::MetaDetails(
            models::meta_details::Selected {
                meta_resource_ref: ResourceRef {
                    resource: "meta".to_string(),
                    type_name: "series".to_string(),
                    id: "st2".to_string(),
                    extra: vec![],
                },
                streams_resource_ref: None,
            },
        )));
        run(runtime.dispatch(&action));
        let state = &runtime.app.read().unwrap().meta_details.to_owned();
        assert_eq!(state.streams_resources.len(), 0, "0 groups");

        let action = Msg::Action(Action::Load(ActionLoad::MetaDetails(
            models::meta_details::Selected {
                meta_resource_ref: ResourceRef {
                    resource: "meta".to_string(),
                    type_name: "series".to_string(),
                    id: "st2".to_string(),
                    extra: vec![],
                },
                streams_resource_ref: Some(ResourceRef {
                    resource: "stream".to_string(),
                    type_name: "series".to_string(),
                    id: "st2v1".to_string(),
                    extra: vec![],
                }),
            },
        )));
        run(runtime.dispatch(&action));
        let state = &runtime.app.read().unwrap().meta_details.to_owned();
        assert_eq!(state.streams_resources.len(), 1, "1 group");
    }

    #[test]
    fn player_stream() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            meta_details: MetaDetails,
            addon_details: AddonDetails,
            player: Player,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // install addon that provides streams
        let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
            models::addon_details::Selected {
                transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
            },
        )));
        run(runtime.dispatch(&addon_details));
        let addon_desc = match runtime
            .app
            .write()
            .unwrap()
            .addon_details
            .addon
            .to_owned()
            .unwrap()
            .content
        {
            Loadable::Ready(x) => x,
            x => panic!("addon not ready, but instead: {:?}", x),
        };
        let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
        run(runtime.dispatch(&install_action));

        let action = Msg::Action(Action::Load(ActionLoad::MetaDetails(
            models::meta_details::Selected {
                meta_resource_ref: ResourceRef {
                    resource: "meta".to_string(),
                    type_name: "series".to_string(),
                    id: "st2".to_string(),
                    extra: vec![],
                },
                streams_resource_ref: Some(ResourceRef {
                    resource: "stream".to_string(),
                    type_name: "series".to_string(),
                    id: "st2v1".to_string(),
                    extra: vec![],
                }),
            },
        )));
        run(runtime.dispatch(&action));

        let first_meta_resource_content =
            match runtime.app.write().unwrap().meta_details.meta_resources[0]
                .to_owned()
                .content
            {
                Loadable::Ready(x) => x,
                x => panic!("content not ready, but instead: {:?}", x),
            };
        let stream = &first_meta_resource_content.videos[0].streams[0];
        let player = Msg::Action(Action::Load(ActionLoad::Player(models::player::Selected {
            stream: stream.to_owned(),
            stream_resource_request: Default::default(),
            meta_resource_request: Default::default(),
            subtitles_resource_ref: Default::default(),
            video_id: Default::default(),
        })));
        run(runtime.dispatch(&player));
        assert_eq!(
            runtime
                .app
                .write()
                .unwrap()
                .player
                .selected
                .to_owned()
                .unwrap()
                .stream,
            stream.to_owned(),
            "stream is the same"
        );

        // testing the unload action
        let unload = Msg::Action(Action::Unload);
        run(runtime.dispatch(&unload));
        assert!(
            match runtime.app.write().unwrap().player.selected {
                None => true,
                _ => false,
            },
            "selected is None"
        );
    }

    #[test]
    fn ctx_and_lib() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            lib_recent: ContinueWatchingPreview,
            notifs: Notifications,
            addon_details: AddonDetails,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        // Log into a user, check if library synced correctly
        run(runtime.dispatch(&Msg::Action(Action::Load(ActionLoad::Ctx))));

        // if this user gets deleted, the test will fail
        // @TODO register a new user instead
        let login_msg = Msg::Action(Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })));
        run(runtime.dispatch(&login_msg));
        // @TODO test if the addon collection is pulled
        let model = &runtime.app.read().unwrap();
        let first_content = model.ctx.profile.to_owned();
        assert!(!model.ctx.library.items.is_empty(), "library has items");
        // LibRecent is "continue watching"
        assert!(!model.lib_recent.lib_items.is_empty(), "has recent items");
        let first_lib = model.ctx.library.to_owned();

        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
        run(runtime.dispatch(&Msg::Action(Action::Load(ActionLoad::Ctx))));
        {
            let ctx = &runtime.app.read().unwrap().ctx;
            assert_eq!(&first_content, &ctx.profile, "content is the same");
            assert_eq!(
                &first_lib, &model.ctx.library,
                "loaded lib is same as synced"
            );
        }

        // Update notifications
        {
            // ¯\_(ツ)_/¯
            // temporary hack (really) until last-videos catalog lands in upstream cinemeta
            // and gets updated for our user
            let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
                models::addon_details::Selected {
                    transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
                },
            )));
            run(runtime.dispatch(&addon_details));

            // remove cinemeta
            runtime.app.write().unwrap().ctx.profile.addons.remove(0);
            let addon_details = Msg::Action(Action::Load(ActionLoad::AddonDetails(
                models::addon_details::Selected {
                    transport_url: "http://127.0.0.1:7001/manifest.json".to_owned(),
                },
            )));
            run(runtime.dispatch(&addon_details));
            let addon_desc = match runtime
                .app
                .write()
                .unwrap()
                .addon_details
                .addon
                .to_owned()
                .unwrap()
                .content
            {
                Loadable::Ready(x) => x,
                x => panic!("addon not ready, but instead: {:?}", x),
            };
            let install_action = Msg::Action(Action::Ctx(ActionCtx::InstallAddon(addon_desc)));
            run(runtime.dispatch(&install_action));

            // we did unspeakable things, now dispatch the load action
            run(runtime.dispatch(&Msg::Action(Action::Load(ActionLoad::Notifications))));
            // ...
            let model = &runtime.app.read().unwrap();
            assert_eq!(model.notifs.groups.len(), 1);
            let meta_items = match &model.notifs.groups[0].content {
                Loadable::Ready(x) => x,
                x => panic!("notifs group not ready, but instead: {:?}", x),
            };
            assert!(meta_items.len() > 1, "should have loaded multiple items");
            // No notifications, cause neither LibItem has .last_vid_released set
            assert!(meta_items.iter().all(|x| x.videos.len() == 0));
        }

        // Logout and expect everything to be reset
        let logout_action = Msg::Action(Action::Ctx(ActionCtx::Logout));
        run(runtime.dispatch(&logout_action));
        {
            let model = &runtime.app.read().unwrap();
            assert!(model.ctx.profile.auth.is_none(), "logged out");
            assert!(model.ctx.profile.addons.len() > 0, "has addons");
            assert!(model.ctx.library.items.is_empty(), "library must be empty");
            assert!(model.lib_recent.lib_items.is_empty(), "is empty");
        }

        // Addon updating in anon mode works
        let zero_ver = semver::Version::new(0, 0, 0);
        {
            let addons = &mut runtime.app.write().unwrap().ctx.profile.addons;
            addons[0].manifest.version = zero_ver.clone();
            addons[0].flags.extra.insert("foo".into(), "bar".into());
            assert_eq!(&addons[0].manifest.version, &zero_ver);
        }
        let update_action = Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI));
        run(runtime.dispatch(&update_action));
        {
            let model = &runtime.app.read().unwrap();
            let first_addon = &model.ctx.profile.addons[0];
            let expected_val = serde_json::Value::String("bar".into());
            assert_ne!(&first_addon.manifest.version, &zero_ver);
            assert_eq!(first_addon.flags.extra.get("foo"), Some(&expected_val));
        }

        // we will now add an item for the anon user
        let item = first_lib.items.values().next().unwrap().to_owned();
        run(runtime.dispatch(&Msg::Internal(Internal::UpdateLibraryItem(item))));

        // take a copy now so we can compare later
        let anon_lib = runtime.app.read().unwrap().ctx.library.to_owned();

        // we will load again to make sure it's persisted
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
        run(runtime.dispatch(&Msg::Action(Action::Load(ActionLoad::Ctx))));
        {
            let ctx = &runtime.app.read().unwrap().ctx;
            assert_eq!(anon_lib, ctx.library);
        }
    }

    // Storage implementation
    // Uses reqwest (asynchronously) for fetch, and a BTreeMap storage
    use lazy_static::*;
    use std::collections::BTreeMap;
    use std::sync::RwLock;
    lazy_static! {
        static ref STORAGE: RwLock<BTreeMap<String, String>> = Default::default();
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
            let mut req =
                reqwest::r#async::Client::new().request(method.to_owned(), &parts.uri.to_string());
            // NOTE: both might be HeaderMap, so maybe there's a better way?
            for (k, v) in parts.headers.iter() {
                req = req.header(k.as_str(), v.as_ref());
            }
            // @TODO add content-type application/json
            // @TODO: if the response code is not 200, return an error related to that
            req = if method.to_owned() == reqwest::Method::GET {
                req
            } else {
                req.json(&body)
            };
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
        fn now() -> DateTime<Utc> {
            Utc::now()
        }
    }
}
