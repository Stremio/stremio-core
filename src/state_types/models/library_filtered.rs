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
                if let LibraryLoadable::Ready(bucket) = &ctx.library {
                    self.types = get_types(bucket);
                    self.items = get_items(bucket, type_name);
                };
                Effects::none()
            }
            Msg::Internal(Internal::LibLoaded(bucket)) => {
                self.types = get_types(bucket);
                if let Some(selected) = &self.selected {
                    self.items = get_items(bucket, selected);
                };
                Effects::none()
            }
            Msg::Event(Event::CtxChanged) | Msg::Event(Event::LibPersisted) => {
                if let LibraryLoadable::Ready(bucket) = &ctx.library {
                    self.types = get_types(bucket);
                    if let Some(selected) = &self.selected {
                        self.items = get_items(bucket, selected);
                    };
                    Effects::none()
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn get_types(bucket: &LibBucket) -> Vec<String> {
    bucket
        .items
        .values()
        .filter(|x| !x.removed)
        .map(|x| x.type_name.to_owned())
        .unique()
        .collect()
}

fn get_items(bucket: &LibBucket, type_name: &String) -> Vec<LibItem> {
    bucket
        .items
        .values()
        .filter(|item| !item.removed && item.type_name.eq(type_name))
        .cloned()
        .collect()
}
