use crate::state_types::msg::actions::ActionUser;
use crate::state_types::msg::ctx_error::CtxError;
use serde_derive::Serialize;

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
