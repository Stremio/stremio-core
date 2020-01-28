use crate::constants::STREAMING_SERVER_URL;
use crate::state_types::models::common::ModelError;
use crate::state_types::models::ctx::user_data::UserDataLoadable;
use crate::state_types::msg::{Action, ActionSettings, Event, Internal, Msg};
use crate::state_types::{Effects, EnvError, Environment, Request, UpdateWithCtx};
use derivative::Derivative;
use futures::future::Future;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
use url_serde;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub app_path: String,
    pub cache_root: String,
    pub cache_size: f64,
    pub bt_max_connections: u64,
    pub bt_handshake_timeout: u64,
    pub bt_request_timeout: u64,
    pub bt_download_speed_soft_limit: f64,
    pub bt_download_speed_hard_limit: f64,
    pub bt_min_peers_for_stable: u64,
    pub server_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum StreamingServerLoadable {
    Offline {
        #[serde(with = "url_serde")]
        url: Url,
    },
    Loading {
        #[serde(with = "url_serde")]
        url: Url,
    },
    Online {
        settings: Settings,
        #[serde(with = "url_serde")]
        base_url: Url,
        #[serde(with = "url_serde")]
        url: Url,
    },
}

impl Default for StreamingServerLoadable {
    fn default() -> Self {
        StreamingServerLoadable::Offline {
            url: Url::parse(STREAMING_SERVER_URL)
                .expect("StreamingServerLoadable url builder cannot fail"),
        }
    }
}

impl StreamingServerLoadable {
    pub fn update<Env: Environment + 'static>(
        &mut self,
        user_data: &UserDataLoadable,
        msg: &Msg,
    ) -> Effects {
        Effects::none().unchanged()
    }
}
