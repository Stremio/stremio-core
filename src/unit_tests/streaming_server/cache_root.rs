use std::any::Any;

use futures::{executor::block_on, future};
use serde_json::json;

use crate::{
    models::{common::Loadable, ctx::Ctx, streaming_server::StreamingServer},
    runtime::{
        msg::{Action, ActionStreamingServer, Internal, Msg},
        Effect, EffectFuture, EnvError, EnvFutureExt, UpdateWithCtx,
    },
    types::{api::SuccessResponse, streaming_server::SettingsResponse, True},
    unit_tests::{TestEnv, FETCH_HANDLER, REQUESTS},
};

fn ready_server(ctx: &Ctx) -> StreamingServer {
    *FETCH_HANDLER.write().unwrap() = Box::new(|_| future::pending().boxed_env());
    let (mut server, _) = StreamingServer::new::<TestEnv>(&ctx.profile);
    REQUESTS.write().unwrap().clear();
    server.settings = Loadable::Ready(
        serde_json::from_value(json!({
            "appPath": "/default", "cacheRoot": "/previous", "serverVersion": "4.21.0",
            "cacheSize": 1024, "btMaxConnections": 55, "btHandshakeTimeout": 20000,
            "btRequestTimeout": 4000, "btDownloadSpeedSoftLimit": 1000,
            "btDownloadSpeedHardLimit": 2000, "btMinPeersForStable": 5,
            "proxyStreamsEnabled": false, "remoteHttps": "", "transcodeProfile": null
        }))
        .unwrap(),
    );
    server
}

fn update(server: &mut StreamingServer, msg: &Msg, ctx: &Ctx) -> crate::runtime::Effects {
    <StreamingServer as UpdateWithCtx<TestEnv>>::update(server, msg, ctx)
}

#[test]
fn cache_root_is_confirmed_without_sending_other_settings() {
    let _env_mutex = TestEnv::reset().unwrap();
    let ctx = Ctx::default();
    let mut server = ready_server(&ctx);
    let mut accepted = server.settings.ready().unwrap().clone();
    accepted.cache_root = "/canonical/folder".to_owned();
    let url = server.selected.transport_url.clone();
    *FETCH_HANDLER.write().unwrap() = Box::new(move |request| {
        assert_eq!(request.url, url.join("settings").unwrap().as_str());
        let response: Box<dyn Any + Send> = match request.method.as_str() {
            "POST" => {
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
                    json!({ "cacheRoot": "/selected/folder" })
                );
                Box::new(SuccessResponse { success: True })
            }
            "GET" => Box::new(SettingsResponse {
                base_url: url.clone(),
                values: accepted.clone(),
                options: vec![],
            }),
            _ => panic!("Unexpected request"),
        };
        future::ok(response).boxed_env()
    });

    let effects = update(
        &mut server,
        &Msg::Action(Action::StreamingServer(
            ActionStreamingServer::UpdateCacheRoot {
                transport_url: ctx.profile.settings.streaming_server_url.clone(),
                cache_root: "/selected/folder".to_owned(),
            },
        )),
        &ctx,
    );
    assert_eq!(server.settings.ready().unwrap().cache_root, "/previous");
    assert_eq!(server.cache_root_update, Some(Loadable::Loading));
    let Effect::Future(EffectFuture::Concurrent(future)) = effects.into_iter().next().unwrap()
    else {
        panic!("Expected settings request");
    };
    update(&mut server, &block_on(future), &ctx);
    assert_eq!(
        server.settings.ready().unwrap().cache_root,
        "/canonical/folder"
    );
    assert_eq!(
        server.cache_root_update,
        Some(Loadable::Ready("/canonical/folder".to_owned()))
    );
    assert_eq!(REQUESTS.read().unwrap().len(), 2);
}

#[test]
fn rejected_cache_root_preserves_settings_and_allows_retry() {
    let _env_mutex = TestEnv::reset().unwrap();
    let ctx = Ctx::default();
    let mut server = ready_server(&ctx);
    let previous_settings = server.settings.clone();
    let error = EnvError::Fetch("Permission denied".to_owned());
    let response_error = error.clone();
    *FETCH_HANDLER.write().unwrap() =
        Box::new(move |_| future::err(response_error.clone()).boxed_env());
    let action = Msg::Action(Action::StreamingServer(
        ActionStreamingServer::UpdateCacheRoot {
            transport_url: server.selected.transport_url.clone(),
            cache_root: "/unwritable".to_owned(),
        },
    ));
    let effects = update(&mut server, &action, &ctx);
    assert!(update(&mut server, &action, &ctx).is_empty());
    let Effect::Future(EffectFuture::Concurrent(future)) = effects.into_iter().next().unwrap()
    else {
        panic!("Expected settings request");
    };
    update(&mut server, &block_on(future), &ctx);
    assert_eq!(server.settings, previous_settings);
    assert_eq!(server.cache_root_update, Some(Loadable::Err(error)));
    assert_eq!(REQUESTS.read().unwrap().len(), 1);
    assert_eq!(update(&mut server, &action, &ctx).len(), 1);
}

#[test]
fn stale_cache_root_results_are_ignored_after_reload_or_server_change() {
    let _env_mutex = TestEnv::reset().unwrap();
    let ctx = Ctx::default();
    let mut server = ready_server(&ctx);
    update(
        &mut server,
        &Msg::Action(Action::StreamingServer(
            ActionStreamingServer::UpdateCacheRoot {
                transport_url: ctx.profile.settings.streaming_server_url.clone(),
                cache_root: "/new".to_owned(),
            },
        )),
        &ctx,
    );
    let result = Msg::Internal(Internal::StreamingServerUpdateCacheRootResult(
        server.selected.transport_url.clone(),
        server.cache_root_update_generation,
        Ok("/new".to_owned()),
    ));
    update(
        &mut server,
        &Msg::Action(Action::StreamingServer(ActionStreamingServer::Reload)),
        &ctx,
    );
    assert!(!update(&mut server, &result, &ctx).has_changed);
    assert_eq!(server.cache_root_update, None);
    server.selected.transport_url = "http://other-server:11470".parse().unwrap();
    assert!(!update(&mut server, &result, &ctx).has_changed);
}

#[test]
fn general_settings_updates_only_send_cache_root_when_it_changes() {
    let _env_mutex = TestEnv::reset().unwrap();
    let ctx = Ctx::default();
    for change_root in [false, true] {
        let mut server = ready_server(&ctx);
        let mut settings = server.settings.ready().unwrap().clone();
        settings.cache_size = Some(2048.0);
        if change_root {
            settings.cache_root = "/new".to_owned();
        }
        update(
            &mut server,
            &Msg::Action(Action::StreamingServer(
                ActionStreamingServer::UpdateSettings(settings),
            )),
            &ctx,
        );
        let requests = REQUESTS.read().unwrap();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(body["cacheSize"], 2048.0);
        assert_eq!(body.get("cacheRoot"), change_root.then_some(&json!("/new")));
    }
}
