use super::{Ctx, LibraryLoadable};
use crate::state_types::*;
use crate::types::{LibBucket, LibItem};
use itertools::Itertools;
use serde_derive::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct LibraryFiltered {
    pub selected: Option<String>,
    pub types: Vec<String>,
    pub items: Vec<LibItem>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibraryFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibItemsByType(type_name))) => {
                self.selected = Some(type_name.to_owned());
                let bucket = match &ctx.library {
                    LibraryLoadable::Ready(bucket) => Some(bucket),
                    _ => None,
                };
                self.types = get_types(bucket);
                self.items = get_items(bucket, Some(type_name));
                Effects::none()
            }
            Msg::Internal(Internal::LibLoaded(bucket)) => {
                self.types = get_types(Some(bucket));
                self.items = get_items(Some(bucket), self.selected.as_ref());
                Effects::none()
            }
            Msg::Event(Event::CtxChanged) | Msg::Event(Event::LibPersisted) => {
                let bucket = match &ctx.library {
                    LibraryLoadable::Ready(bucket) => Some(bucket),
                    _ => None,
                };
                self.types = get_types(bucket);
                self.items = get_items(bucket, self.selected.as_ref());
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn get_types(bucket: Option<&LibBucket>) -> Vec<String> {
    if let Some(bucket) = bucket {
        bucket
            .items
            .values()
            .filter(|x| !x.removed)
            .map(|x| x.type_name.to_owned())
            .unique()
            .collect()
    } else {
        Vec::new()
    }
}

fn get_items(bucket: Option<&LibBucket>, type_name: Option<&String>) -> Vec<LibItem> {
    if bucket.is_some() && type_name.is_some() {
        bucket
            .unwrap()
            .items
            .values()
            .filter(|item| !item.removed && item.type_name.eq(type_name.unwrap()))
            .cloned()
            .collect()
    } else {
        Vec::new()
    }
}
