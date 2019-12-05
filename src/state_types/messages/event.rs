use crate::state_types::messages::action::ActionUser;
use crate::state_types::EnvError;
use crate::types::api::APIErr;
use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "err", content = "args")]
pub enum CtxError {
    API(APIErr),
    Env { message: String },
}

impl From<APIErr> for CtxError {
    fn from(e: APIErr) -> Self {
        CtxError::API(e)
    }
}

impl From<EnvError> for CtxError {
    fn from(e: EnvError) -> Self {
        CtxError::Env {
            message: e.to_string(),
        }
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
