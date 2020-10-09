use crate::models::common::eq_update;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::library::{LibraryBucket, LibraryItem};
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
    fn predicate(library_item: &LibraryItem) -> bool;
}

pub enum ContinueWatchingFilter {}

impl LibraryFilter for ContinueWatchingFilter {
    fn predicate(library_item: &LibraryItem) -> bool {
        library_item.is_in_continue_watching()
    }
}

pub enum NotRemovedFilter {}

impl LibraryFilter for NotRemovedFilter {
    fn predicate(library_item: &LibraryItem) -> bool {
        !library_item.removed
    }
}

#[derive(Derivative, Serialize)]
#[derivative(Default)]
pub struct LibraryWithFilters<F> {
    pub selected: Option<Selected>,
    pub type_names: Vec<String>,
    pub library_items: Vec<LibraryItem>,
    pub filter: PhantomData<F>,
}

impl<E, F> UpdateWithCtx<Ctx<E>> for LibraryWithFilters<F>
where
    E: Env + 'static,
    F: LibraryFilter,
{
    fn update(&mut self, msg: &Msg, ctx: &Ctx<E>) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryWithFilters(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let library_items_effects = library_items_update::<F>(
                    &mut self.library_items,
                    &self.selected,
                    &ctx.library,
                );
                selected_effects.join(library_items_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let library_items_effects = library_items_update::<F>(
                    &mut self.library_items,
                    &self.selected,
                    &ctx.library,
                );
                selected_effects.join(library_items_effects)
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                let type_names_effects = type_names_update::<F>(&mut self.type_names, &ctx.library);
                let library_items_effects = library_items_update::<F>(
                    &mut self.library_items,
                    &self.selected,
                    &ctx.library,
                );
                type_names_effects.join(library_items_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn type_names_update<F: LibraryFilter>(
    type_names: &mut Vec<String>,
    library: &LibraryBucket,
) -> Effects {
    let next_type_names = library
        .items
        .values()
        .filter(|library_item| F::predicate(library_item))
        .map(|library_item| library_item.type_name.to_owned())
        .unique()
        .collect::<Vec<_>>();
    if *type_names != next_type_names {
        *type_names = next_type_names;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn library_items_update<F: LibraryFilter>(
    library_items: &mut Vec<LibraryItem>,
    selected: &Option<Selected>,
    library: &LibraryBucket,
) -> Effects {
    let next_library_items = match selected {
        Some(selected) => library
            .items
            .values()
            .filter(|library_item| F::predicate(library_item))
            .filter(|library_item| match &selected.type_name {
                Some(type_name) => *type_name == library_item.type_name,
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
    if *library_items != next_library_items {
        *library_items = next_library_items;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
