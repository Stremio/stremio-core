use crate::state_types::models::common::eq_update;
use crate::state_types::models::ctx::library_loadable::LibraryLoadable;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionLoad, Internal, Msg};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::{LibItem, UID};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "lowercase")]
pub enum SortProp {
    #[derivative(Default)]
    CTime,
    Year,
    Name,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    type_name: String,
    #[serde(default)]
    sort_prop: SortProp,
}

#[derive(Derivative, Debug, Clone, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(tag = "type")]
pub enum LibraryState {
    Loading {
        uid: UID,
    },
    #[derivative(Default)]
    Ready {
        uid: UID,
    },
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct LibraryFiltered {
    pub selected: Option<Selected>,
    pub library_state: LibraryState,
    pub type_names: Vec<String>,
    pub lib_items: Vec<LibItem>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibraryFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryFiltered(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let lib_items_effects =
                    lib_items_update(&mut self.lib_items, &self.selected, &ctx.library);
                selected_effects.join(lib_items_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let lib_items_effects =
                    lib_items_update(&mut self.lib_items, &self.selected, &ctx.library);
                selected_effects.join(lib_items_effects)
            }
            Msg::Internal(Internal::LibraryChanged) => {
                let library_state_effects =
                    library_state_update(&mut self.library_state, &ctx.library);
                let type_names_effects = type_names_update(&mut self.type_names, &ctx.library);
                let lib_items_effects =
                    lib_items_update(&mut self.lib_items, &self.selected, &ctx.library);
                library_state_effects
                    .join(type_names_effects)
                    .join(lib_items_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn library_state_update(library_state: &mut LibraryState, library: &LibraryLoadable) -> Effects {
    let next_library_state = match library {
        LibraryLoadable::Loading(uid, _) => LibraryState::Loading {
            uid: uid.to_owned(),
        },
        LibraryLoadable::Ready(bucket) => LibraryState::Ready {
            uid: bucket.uid.to_owned(),
        },
    };
    if next_library_state.ne(library_state) {
        *library_state = next_library_state;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn type_names_update(type_names: &mut Vec<String>, library: &LibraryLoadable) -> Effects {
    let next_type_names = match library {
        LibraryLoadable::Ready(bucket) => bucket
            .items
            .values()
            .filter(|lib_item| !lib_item.removed)
            .map(|lib_item| lib_item.type_name.to_owned())
            .unique()
            .collect(),
        _ => vec![],
    };
    if next_type_names.ne(type_names) {
        *type_names = next_type_names;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn lib_items_update(
    lib_items: &mut Vec<LibItem>,
    selected: &Option<Selected>,
    library: &LibraryLoadable,
) -> Effects {
    let next_lib_items = match (selected, library) {
        (Some(selected), LibraryLoadable::Ready(bucket)) => bucket
            .items
            .values()
            .filter(|lib_item| !lib_item.removed && lib_item.type_name.eq(&selected.type_name))
            .sorted_by(|a, b| match &selected.sort_prop {
                SortProp::Year => b.year.cmp(&a.year),
                SortProp::Name => a.name.cmp(&b.name),
                SortProp::CTime => b.ctime.cmp(&a.ctime),
            })
            .cloned()
            .collect(),
        _ => vec![],
    };
    if next_lib_items.ne(lib_items) {
        *lib_items = next_lib_items;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
