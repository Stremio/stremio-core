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
pub struct Settings {}

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

impl StreamingServerLoadable {
    pub fn update<Env: Environment + 'static>(
        &mut self,
        user_data: &UserDataLoadable,
        msg: &Msg,
    ) -> Effects {
        Effects::none().unchanged()
    }
}
