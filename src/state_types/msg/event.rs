use crate::state_types::msg::action::ActionUser;
use crate::types::api::APIErr;
use serde_derive::Serialize;
use std::error::Error;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "err", content = "args")]
pub enum CtxError {
    API(APIErr),
    Env(String),
}

impl From<APIErr> for CtxError {
    fn from(e: APIErr) -> Self {
        CtxError::API(e)
    }
}

impl From<Box<dyn Error>> for CtxError {
    fn from(e: Box<dyn Error>) -> Self {
        CtxError::Env(e.to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    CtxSaved,
    CtxChanged,
    CtxAddonsChangedFromPull,
    CtxAddonsPushed,
    CtxFatal(CtxError),
    CtxActionErr(ActionUser, CtxError),
    LibPersisted,
    LibPushed,
    LibFatal(CtxError),
    SettingsStoreError(String),
}
