use super::msg::Internal::*;
use super::msg::*;
use crate::state_types::{Context, Container};
use crate::types::addons::*;
use crate::types::{MetaPreview, Stream};
use serde_derive::*;
use std::sync::Arc;

const UNEXPECTED_RESP_MSG: &str = "unexpected ResourceResponse";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, M> {
    Loading,
    ReadyEmpty,
    Ready(R),
    Message(M),
}
impl<R, M> Loadable<R, M> {
    pub fn is_ready(&self) -> bool {
        match self {
            Loadable::Ready(_) | Loadable::ReadyEmpty => true,
            _ => false,
        }
    }
}

macro_rules! result_to_loadable {
    ($r:ident, $m:ident, $p:pat, $e:expr) => {
        match $r {
            Ok($p) if $m.len() == 0 => Loadable::ReadyEmpty,
            Ok($p) => Loadable::Ready($e),
            Ok(_) => Loadable::Message(UNEXPECTED_RESP_MSG.to_owned()),
            Err(e) => Loadable::Message(e.to_owned()),
        }
    };
}

// @TODO better type for Message
pub type Message = String;

type LoadableItems = Loadable<Vec<MetaPreview>, Message>;
// Here's why we use Arc: https://gist.github.com/Ivshti/7ddf0fa6c7d50b5211d8f771241f64ab
type Group = Arc<(ResourceRequest, LoadableItems)>;
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CatalogGrouped {
    pub groups: Vec<Group>,
}
impl CatalogGrouped {
    pub fn new() -> CatalogGrouped {
        CatalogGrouped { groups: vec![] }
    }
}
impl Container for CatalogGrouped {
    fn update(&self, msg: &Msg) -> Option<Box<Self>> {
        catalogs_reducer(&self, msg)
    }
}

fn catalogs_reducer(state: &CatalogGrouped, msg: &Msg) -> Option<Box<CatalogGrouped>> {
    match msg {
        Msg::Internal(LoadWithCtx(
            Context { addons, .. },
            load_action @ ActionLoad::CatalogGrouped { .. },
        )) => {
            if let Some(aggr_req) = load_action.addon_aggr_req() {
                let groups = aggr_req
                    .plan(&addons)
                    .iter()
                    .map(|req| Arc::new((req.to_owned(), Loadable::Loading)))
                    .collect();
                Some(Box::new(CatalogGrouped { groups }))
            } else {
                None
            }
        }
        Msg::Internal(AddonResponse(req, result)) => {
            if let Some(idx) = state.groups.iter().position(|g| &g.0 == req) {
                let mut groups = state.groups.to_owned();
                let group_content = result_to_loadable!(
                    result,
                    metas,
                    ResourceResponse::Metas { metas },
                    metas.to_owned()
                );
                groups[idx] = Arc::new((req.to_owned(), group_content));
                Some(Box::new(CatalogGrouped { groups }))
            } else {
                None
            }
        }
        _ => {
            // Doesn't mutate
            None
        }
    }
}

//
// Filtered catalogs
// @TODO extra (filters)
// @TODO pagination
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CatalogFiltered {
    pub item_pages: Vec<LoadableItems>,
    pub catalogs: Vec<ManifestCatalog>,
    pub selected: Option<ResourceRequest>,
    // @TODO catalogs to be { is_selected, resource_ref, name, type }
    // is_selected will be whether the resource_ref matches selected, excluding the page
    // @TODO: extra (filters)
    // @TODO pagination; this can be done by incrementing skip in the ResourceRef when requesting
    // the next page; in LoadWithCtx, when we see that the request is for the next page, we add
    // another entry to item_pages
    // @TODO consider having `types` as well, with `is_selected`; this will just be an aggregated
    // view of `catalogs` for convenience
}
impl CatalogFiltered {
    pub fn new() -> CatalogFiltered {
        CatalogFiltered {
            item_pages: vec![],
            catalogs: vec![],
            selected: None,
        }
    }
}
impl Container for CatalogFiltered {
    fn update(&self, msg: &Msg) -> Option<Box<Self>> {
        match msg {
            Msg::Internal(LoadWithCtx(
                Context { addons, .. },
                ActionLoad::CatalogFiltered { resource_req },
            )) => {
                //dbg!(&addons);
                //dbg!(&resource_req);
                // @TODO pagination
                let catalogs = addons
                    .iter()
                    .map(|a| &a.manifest.catalogs)
                    .cloned()
                    .flatten()
                    .filter(|cat| cat.is_extra_supported(&[]))
                    .collect();
                Some(Box::new(CatalogFiltered {
                    catalogs,
                    item_pages: vec![Loadable::Loading],
                    selected: Some(resource_req.to_owned()),
                }))
            }
            Msg::Internal(AddonResponse(req, result))
                if Some(req) == self.selected.as_ref()
                    && self.item_pages.last() == Some(&Loadable::Loading) =>
            {
                // @TODO pagination
                let mut new_state = self.to_owned();
                new_state.item_pages[0] = result_to_loadable!(
                    result,
                    metas,
                    ResourceResponse::Metas { metas },
                    metas.to_owned()
                );
                Some(Box::new(new_state))
            }
            _ => None,
        }
    }
}

// @TODO split this out
// @TODO streams should contain info about which addon the response is from
pub type LoadableStreams = Loadable<Vec<Stream>, Message>;
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Streams {
    pub groups: Vec<(ResourceRequest, LoadableStreams)>,
}
impl Streams {
    pub fn new() -> Streams {
        Streams { groups: vec![] }
    }
}
impl Container for Streams {
    fn update(&self, msg: &Msg) -> Option<Box<Self>> {
        match msg {
            Msg::Internal(LoadWithCtx(
                Context { addons, .. },
                load_action @ ActionLoad::Streams { .. },
            )) => {
                if let Some(aggr_req) = load_action.addon_aggr_req() {
                    let groups = aggr_req
                        .plan(&addons)
                        .iter()
                        .map(|req| (req.to_owned(), Loadable::Loading))
                        .collect();
                    Some(Box::new(Streams { groups }))
                } else {
                    None
                }
            }
            Msg::Internal(AddonResponse(req, result)) => {
                if let Some(idx) = self.groups.iter().position(|g| &g.0 == req) {
                    let mut groups = self.groups.to_owned();
                    groups[idx].1 = result_to_loadable!(
                        result,
                        streams,
                        ResourceResponse::Streams { streams },
                        streams.to_owned()
                    );
                    Some(Box::new(Streams { groups }))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
