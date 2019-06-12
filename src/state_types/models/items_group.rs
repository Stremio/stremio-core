use crate::state_types::*;
use crate::types::addons::{Descriptor, ResourceRequest, ResourceResponse};
use serde_derive::*;
use std::convert::TryInto;

pub const UNEXPECTED_RESP_MSG: &str = "unexpected ResourceResponse";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, E> {
    Loading,
    Ready(R),
    Err(E),
}

#[derive(Debug, Serialize, Clone)]
pub struct ItemsGroup<T> {
    req: ResourceRequest,
    pub content: Loadable<T, String>,
}
impl<T> Group for ItemsGroup<T>
where
    ResourceResponse: TryInto<T>,
{
    fn new(_: &Descriptor, req: ResourceRequest) -> Self {
        ItemsGroup {
            req,
            content: Loadable::Loading,
        }
    }
    fn update(&mut self, res: &Result<ResourceResponse, EnvError>) {
        self.content = match res {
            Ok(resp) => match resp.to_owned().try_into() {
                Ok(x) => Loadable::Ready(x),
                Err(_) => Loadable::Err(UNEXPECTED_RESP_MSG.to_string()),
            },
            Err(e) => Loadable::Err(e.to_string()),
        };
    }
    fn addon_req(&self) -> &ResourceRequest {
        &self.req
    }
}
