use crate::models::common::eq_update;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Environment, UpdateWithCtx};
use crate::types::library::{LibBucket, LibItem};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    LastWatched,
    TimesWatched,
    Name,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub type_name: Option<String>,
    pub sort: Sort,
}

pub trait LibraryFilter {
    fn predicate(lib_item: &LibItem) -> bool;
}

pub enum ContinueWatchingFilter {}

impl LibraryFilter for ContinueWatchingFilter {
    fn predicate(lib_item: &LibItem) -> bool {
        lib_item.is_in_continue_watching()
    }
}

pub enum NotRemovedFilter {}

impl LibraryFilter for NotRemovedFilter {
    fn predicate(lib_item: &LibItem) -> bool {
        !lib_item.removed
    }
}

#[derive(Derivative, Serialize)]
#[derivative(Default(bound = ""))]
pub struct LibraryWithFilters<F> {
    pub selected: Option<Selected>,
    pub type_names: Vec<String>,
    pub lib_items: Vec<LibItem>,
    pub filter: PhantomData<F>,
}

impl<Env, F> UpdateWithCtx<Ctx<Env>> for LibraryWithFilters<F>
where
    Env: Environment + 'static,
    F: LibraryFilter,
{
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryWithFilters(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let lib_items_effects =
                    lib_items_update::<F>(&mut self.lib_items, &self.selected, &ctx.library);
                selected_effects.join(lib_items_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let lib_items_effects =
                    lib_items_update::<F>(&mut self.lib_items, &self.selected, &ctx.library);
                selected_effects.join(lib_items_effects)
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                let type_names_effects = type_names_update::<F>(&mut self.type_names, &ctx.library);
                let lib_items_effects =
                    lib_items_update::<F>(&mut self.lib_items, &self.selected, &ctx.library);
                type_names_effects.join(lib_items_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn type_names_update<F: LibraryFilter>(
    type_names: &mut Vec<String>,
    library: &LibBucket,
) -> Effects {
    let next_type_names = library
        .items
        .values()
        .filter(|lib_item| F::predicate(lib_item))
        .map(|lib_item| lib_item.type_name.to_owned())
        .unique()
        .collect::<Vec<_>>();
    if *type_names != next_type_names {
        *type_names = next_type_names;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn lib_items_update<F: LibraryFilter>(
    lib_items: &mut Vec<LibItem>,
    selected: &Option<Selected>,
    library: &LibBucket,
) -> Effects {
    let next_lib_items = match selected {
        Some(selected) => library
            .items
            .values()
            .filter(|lib_item| F::predicate(lib_item))
            .filter(|lib_item| match &selected.type_name {
                Some(type_name) => *type_name == lib_item.type_name,
                None => true,
            })
            .sorted_by(|a, b| match &selected.sort {
                Sort::LastWatched => b.state.last_watched.cmp(&a.state.last_watched),
                Sort::TimesWatched => b.state.times_watched.cmp(&a.state.times_watched),
                Sort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            })
            .cloned()
            .collect(),
        _ => vec![],
    };
    if *lib_items != next_lib_items {
        *lib_items = next_lib_items;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
