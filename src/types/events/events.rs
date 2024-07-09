use serde::Serialize;

use crate::{
    models::{common::Loadable, ctx::CtxError},
    types::api::{GetModalResponse, GetNotificationResponse},
};

#[derive(Default, PartialEq, Eq, Serialize, Clone, Debug)]
pub struct Events {
    pub modal: Loadable<Option<GetModalResponse>, CtxError>,
    pub notification: Loadable<Option<GetNotificationResponse>, CtxError>,
}
