use super::{Ctx, LibraryLoadable};
use crate::state_types::*;
use crate::types::{LibBucket, LibItem, UID};
use derivative::*;
use itertools::Itertools;
use serde_derive::*;

#[derive(Derivative, Debug, Clone, Serialize, PartialEq)]
#[derivative(Default)]
#[serde(tag = "type", content = "content")]
pub enum LibraryState {
    #[derivative(Default)]
    NotLoaded,
    Loading(UID),
    Ready(UID),
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LibraryFiltered {
    pub library_state: LibraryState,
    pub selected: Option<String>,
    pub type_names: Vec<String>,
    pub items: Vec<LibItem>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibraryFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryFiltered { type_name })) => {
                let bucket = match &ctx.library {
                    LibraryLoadable::Ready(bucket) => Some(bucket),
                    _ => None,
                };
                let (selected, selected_effects) =
                    selected_reducer(self.selected.as_ref(), Some(&type_name));
                let (type_names, type_names_effects) = type_names_reducer(&self.type_names, bucket);
                let (items, items_effects) =
                    lib_items_reducer(&self.items, bucket, Some(&type_name));
                self.selected = selected;
                self.type_names = type_names;
                self.items = items;
                selected_effects
                    .join(type_names_effects)
                    .join(items_effects)
            }
            Msg::Event(Event::CtxChanged)
            | Msg::Event(Event::LibPersisted)
            | Msg::Internal(Internal::LibLoaded(_)) => {
                let bucket = match &ctx.library {
                    LibraryLoadable::Ready(bucket) => Some(bucket),
                    _ => None,
                };
                let (library_state, library_state_effects) =
                    library_state_reducer(&self.library_state, &ctx.library);
                let (type_names, type_names_effects) = type_names_reducer(&self.type_names, bucket);
                let (items, items_effects) =
                    lib_items_reducer(&self.items, bucket, self.selected.as_ref());
                self.library_state = library_state;
                self.type_names = type_names;
                self.items = items;
                library_state_effects
                    .join(type_names_effects)
                    .join(items_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn library_state_reducer(
    prev_library_state: &LibraryState,
    library: &LibraryLoadable,
) -> (LibraryState, Effects) {
    let next_library_state = match library {
        LibraryLoadable::Ready(bucket) => LibraryState::Ready(bucket.uid.to_owned()),
        LibraryLoadable::Loading(uid) => LibraryState::Loading(uid.to_owned()),
        LibraryLoadable::NotLoaded => LibraryState::NotLoaded,
    };
    if prev_library_state.eq(&next_library_state) {
        (prev_library_state.to_owned(), Effects::none().unchanged())
    } else {
        (next_library_state, Effects::none())
    }
}

fn selected_reducer(
    prev_selected: Option<&String>,
    type_name: Option<&String>,
) -> (Option<String>, Effects) {
    let next_selected = match type_name {
        Some(type_name) => Some(type_name),
        None => None,
    };
    if prev_selected.eq(&next_selected) {
        (prev_selected.cloned(), Effects::none().unchanged())
    } else {
        (next_selected.cloned(), Effects::none())
    }
}

fn type_names_reducer(
    prev_type_names: &Vec<String>,
    bucket: Option<&LibBucket>,
) -> (Vec<String>, Effects) {
    let next_type_names = match bucket {
        Some(bucket) => bucket
            .items
            .values()
            .filter(|x| !x.removed)
            .map(|x| x.type_name.to_owned())
            .unique()
            .collect(),
        _ => Vec::new(),
    };
    if prev_type_names.iter().eq(next_type_names.iter()) {
        (prev_type_names.to_owned(), Effects::none().unchanged())
    } else {
        (next_type_names, Effects::none())
    }
}

fn lib_items_reducer(
    prev_lib_items: &Vec<LibItem>,
    bucket: Option<&LibBucket>,
    type_name: Option<&String>,
) -> (Vec<LibItem>, Effects) {
    let next_lib_items = match (bucket, type_name) {
        (Some(bucket), Some(type_name)) => bucket
            .items
            .values()
            .filter(|item| !item.removed && item.type_name.eq(type_name))
            .cloned()
            .collect(),
        _ => Vec::new(),
    };
    if prev_lib_items.iter().eq(next_lib_items.iter()) {
        (prev_lib_items.to_owned(), Effects::none().unchanged())
    } else {
        (next_lib_items, Effects::none())
    }
}
