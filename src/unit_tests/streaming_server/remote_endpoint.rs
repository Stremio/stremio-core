use std::any::Any;

use futures::future;
use stremio_derive::Model;
use url::Url;

use crate::{
    models::{
        ctx::Ctx,
        streaming_server::{PlaybackDevice, StreamingServer},
    },
    runtime::{
        msg::{Action, ActionStreamingServer},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        api::SuccessResponse,
        profile::{Auth, AuthKey, Profile},
        streaming_server::{
            DeviceInfo, GetHTTPSResponse, NetworkInfo, Settings as StreamingServerSettings,
            SettingsResponse,
        },
        True,
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER},
};

const STREAMING_SERVER_URL: &str = "http://127.0.0.1:11470";
const STREAMING_SERVER_SETTINGS: StreamingServerSettings = StreamingServerSettings {
    app_path: String::new(),
    cache_root: String::new(),
    server_version: String::new(),
    cache_size: None,
    bt_max_connections: 0,
    bt_handshake_timeout: 0,
    bt_request_timeout: 0,
    bt_download_speed_soft_limit: 0.0,
    bt_download_speed_hard_limit: 0.0,
    bt_min_peers_for_stable: 0,
    proxy_streams_enabled: false,
    remote_https: None,
    transcode_profile: None,
};

const AVAILABLE_INTERFACE: &str = "192.168.0.10";

#[test]
fn remote_endpoint() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        streaming_server: StreamingServer,
    }

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, method, .. }
                if method == "GET" && url == "http://127.0.0.1:11470/settings" =>
            {
                future::ok(Box::new(SettingsResponse {
                    base_url: Url::parse(STREAMING_SERVER_URL).unwrap(),
                    values: STREAMING_SERVER_SETTINGS,
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, method, .. }
                if method == "POST" && url == "http://127.0.0.1:11470/settings" =>
            {
                future::ok(Box::new(SuccessResponse { success: True }) as Box<dyn Any + Send>)
                    .boxed_env()
            }
            Request { url, .. } if url == "http://127.0.0.1:11470/casting" => {
                future::ok(Box::<Vec<PlaybackDevice>>::default() as Box<dyn Any + Send>).boxed_env()
            }
            Request { url, .. } if url == "http://127.0.0.1:11470/network-info" => {
                future::ok(Box::new(NetworkInfo {
                    available_interfaces: vec![AVAILABLE_INTERFACE.to_string()],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, .. } if url == "http://127.0.0.1:11470/device-info" => {
                future::ok(Box::new(DeviceInfo {
                    available_hardware_accelerations: vec![],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, .. } if url.starts_with("http://127.0.0.1:11470/get-https") => {
                future::ok(Box::new(GetHTTPSResponse {
                    ip_address: AVAILABLE_INTERFACE.to_string(),
                    domain: "https://stremio.com".to_string(),
                    port: 3333,
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let profile = Profile {
        auth: Some(Auth {
            key: AuthKey("auth_key".to_owned()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let (streaming_server, ..) = StreamingServer::new::<TestEnv>(&profile);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile,
                ..Default::default()
            },
            streaming_server,
        },
        vec![],
        1000,
    );

    assert!(
        runtime
            .model()
            .unwrap()
            .streaming_server
            .remote_url
            .is_none(),
        "Remote url should not be set"
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::StreamingServer(ActionStreamingServer::Reload),
        });
    });

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::StreamingServer(ActionStreamingServer::UpdateSettings(
                StreamingServerSettings {
                    remote_https: Some(AVAILABLE_INTERFACE.to_string()),
                    ..STREAMING_SERVER_SETTINGS
                },
            )),
        });
    });

    assert!(
        runtime
            .model()
            .unwrap()
            .streaming_server
            .remote_url
            .is_some(),
        "Remote url should be set"
    );
}
