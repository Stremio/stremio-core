use crate::constants::TYPE_PRIORITIES;
use crate::models::common::{compare_with_priorities, eq_update};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::library::{LibraryBucket, LibraryItem};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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

#[derive(Derivative, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    #[derivative(Default)]
    LastWatched,
    Name,
    TimesWatched,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryRequest {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[serde(default)]
    pub sort: Sort,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub request: LibraryRequest,
}

#[derive(PartialEq, Serialize)]
pub struct SelectableType {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub selected: bool,
    pub request: LibraryRequest,
}

#[derive(PartialEq, Serialize)]
pub struct SelectableSort {
    pub sort: Sort,
    pub selected: bool,
    pub request: LibraryRequest,
}

#[derive(Default, PartialEq, Serialize)]
pub struct Selectable {
    pub types: Vec<SelectableType>,
    pub sorts: Vec<SelectableSort>,
}

#[derive(Derivative, Serialize)]
#[derivative(Default)]
pub struct LibraryWithFilters<F> {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub library_items: Vec<LibraryItem>,
    pub filter: PhantomData<F>,
}

impl<F: LibraryFilter> LibraryWithFilters<F> {
    pub fn new(library: &LibraryBucket) -> (Self, Effects) {
        let selected = None;
        let mut selectable = Selectable::default();
        let effects = selectable_update::<F>(&mut selectable, &selected, &library);
        (
            LibraryWithFilters {
                selectable,
                selected,
                library_items: vec![],
                filter: PhantomData,
            },
            effects.unchanged(),
        )
    }
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
                let selectable_effects =
                    selectable_update::<F>(&mut self.selectable, &self.selected, &ctx.library);
                let library_items_effects = library_items_update::<F>(
                    &mut self.library_items,
                    &self.selected,
                    &ctx.library,
                );
                selected_effects
                    .join(selectable_effects)
                    .join(library_items_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let selectable_effects =
                    selectable_update::<F>(&mut self.selectable, &self.selected, &ctx.library);
                let library_items_effects = library_items_update::<F>(
                    &mut self.library_items,
                    &self.selected,
                    &ctx.library,
                );
                selected_effects
                    .join(selectable_effects)
                    .join(library_items_effects)
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                let selectable_effects =
                    selectable_update::<F>(&mut self.selectable, &self.selected, &ctx.library);
                let library_items_effects = library_items_update::<F>(
                    &mut self.library_items,
                    &self.selected,
                    &ctx.library,
                );
                selectable_effects.join(library_items_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn selectable_update<F: LibraryFilter>(
    selectable: &mut Selectable,
    selected: &Option<Selected>,
    library: &LibraryBucket,
) -> Effects {
    let selectable_types = library
        .items
        .values()
        .filter(|library_item| F::predicate(library_item))
        .map(|library_item| &library_item.type_)
        .unique()
        .sorted_by(|a, b| compare_with_priorities(a.as_str(), b.as_str(), &*TYPE_PRIORITIES))
        .rev()
        .cloned()
        .map(Some)
        .map(|type_| SelectableType {
            type_: type_.to_owned(),
            request: LibraryRequest {
                type_: type_.to_owned(),
                sort: selected
                    .as_ref()
                    .map(|selected| selected.request.sort.to_owned())
                    .unwrap_or_default(),
            },
            selected: selected
                .as_ref()
                .map(|selected| selected.request.type_ == type_)
                .unwrap_or_default(),
        })
        .collect();
    let selectable_sorts = Sort::iter()
        .map(|sort| SelectableSort {
            sort: sort.to_owned(),
            request: LibraryRequest {
                type_: selected
                    .as_ref()
                    .and_then(|selected| selected.request.type_.to_owned()),
                sort: sort.to_owned(),
            },
            selected: selected
                .as_ref()
                .map(|selected| selected.request.sort == sort)
                .unwrap_or_default(),
        })
        .collect();
    let next_selectable = Selectable {
        types: selectable_types,
        sorts: selectable_sorts,
    };
    if *selectable != next_selectable {
        *selectable = next_selectable;
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
            .filter(|library_item| match &selected.request.type_ {
                Some(type_) => library_item.type_ == *type_,
                None => true,
            })
            .sorted_by(|a, b| match &selected.request.sort {
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
