use crate::state_types::models::common::eq_update;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionLoad, Internal, Msg};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::{LibBucket, LibItem};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    LastWatched,
    TimesWatched,
    Name,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    type_name: Option<String>,
    sort: Sort,
    continue_watching: bool,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct LibraryWithFilters {
    pub selected: Option<Selected>,
    pub type_names: Vec<String>,
    pub lib_items: Vec<LibItem>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibraryWithFilters {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryWithFilters(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let lib_items_effects =
                    lib_items_update(&mut self.lib_items, &self.selected, ctx.library());
                selected_effects.join(lib_items_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let lib_items_effects =
                    lib_items_update(&mut self.lib_items, &self.selected, ctx.library());
                selected_effects.join(lib_items_effects)
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                let type_names_effects = type_names_update(&mut self.type_names, ctx.library());
                let lib_items_effects =
                    lib_items_update(&mut self.lib_items, &self.selected, ctx.library());
                type_names_effects.join(lib_items_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn type_names_update(type_names: &mut Vec<String>, library: &LibBucket) -> Effects {
    let next_type_names = library
        .items
        .values()
        .filter(|lib_item| !lib_item.removed)
        .map(|lib_item| lib_item.type_name.to_owned())
        .unique()
        .collect::<Vec<_>>();
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
    library: &LibBucket,
) -> Effects {
    let next_lib_items = match selected {
        Some(selected) => library
            .items
            .values()
            .filter(|lib_item| !lib_item.removed)
            .filter(|lib_item| match &selected.type_name {
                Some(type_name) => lib_item.type_name.eq(type_name),
                None => true,
            })
            .filter(|lib_item| !selected.continue_watching || lib_item.state.time_offset > 0)
            .sorted_by(|a, b| match &selected.sort {
                Sort::LastWatched => a.state.last_watched.cmp(&b.state.last_watched),
                Sort::TimesWatched => a.state.times_watched.cmp(&b.state.times_watched),
                Sort::Name => a.name.cmp(&b.name),
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
