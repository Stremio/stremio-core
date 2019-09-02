use super::CatalogError;
use crate::state_types::*;
use crate::types::addons::{Descriptor, ResourceRequest, ResourceResponse};
use serde_derive::*;
use std::convert::TryInto;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, E> {
    Loading,
    Ready(R),
    Err(E),
}
impl<R, E> Default for Loadable<R, E> {
    fn default() -> Self {
        Loadable::Loading
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ItemsGroup<T> {
    req: ResourceRequest,
    pub content: Loadable<T, CatalogError>,
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
                Err(_) => Loadable::Err(CatalogError::UnexpectedResp),
            },
            Err(e) => Loadable::Err(CatalogError::Other(e.to_string())),
        };
    }
    fn addon_req(&self) -> &ResourceRequest {
        &self.req
    }
}
