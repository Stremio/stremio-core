use std::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering},
};

use futures::future;
use stremio_derive::Model;
use url::Url;

use crate::{
    models::{
        common::Loadable,
        ctx::Ctx,
        streaming_server::{PlaybackDevice, StreamingServer},
    },
    runtime::{
        msg::{Action, ActionStreamingServer},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        profile::Profile,
        streaming_server::{
            DeviceInfo, NetworkInfo, Settings as StreamingServerSettings, SettingsResponse,
        },
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
fn refresh_playback_devices_updates_ready_state() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        streaming_server: StreamingServer,
    }

    static CASTING_REQUESTS: AtomicUsize = AtomicUsize::new(0);
    CASTING_REQUESTS.store(0, Ordering::SeqCst);

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, method, .. }
                if method == "GET" && url == "http://127.0.0.1:11470/settings" =>
            {
                future::ok(Box::new(SettingsResponse {
                    base_url: Url::parse(STREAMING_SERVER_URL).unwrap(),
                    values: STREAMING_SERVER_SETTINGS,
                    options: vec![],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, .. } if url == "http://127.0.0.1:11470/casting" => {
                let request_index = CASTING_REQUESTS.fetch_add(1, Ordering::SeqCst);
                let devices = match request_index {
                    0 | 1 => vec![],
                    _ => vec![PlaybackDevice {
                        id: "chromecast-device".to_owned(),
                        name: "Living Room TV".to_owned(),
                        r#type: "chromecast".to_owned(),
                    }],
                };

                future::ok(Box::new(devices) as Box<dyn Any + Send>).boxed_env()
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
            _ => default_fetch_handler(request),
        }
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let profile = Profile::default();
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

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::StreamingServer(ActionStreamingServer::Reload),
        });
    });

    assert_eq!(
        runtime.model().unwrap().streaming_server.playback_devices,
        Loadable::Ready(vec![]),
        "Initial playback devices should be ready but empty"
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::StreamingServer(ActionStreamingServer::RefreshPlaybackDevices),
        });
    });

    assert_eq!(
        runtime.model().unwrap().streaming_server.playback_devices,
        Loadable::Ready(vec![PlaybackDevice {
            id: "chromecast-device".to_owned(),
            name: "Living Room TV".to_owned(),
            r#type: "chromecast".to_owned(),
        }]),
        "RefreshPlaybackDevices should update an already-ready device list"
    );
}
