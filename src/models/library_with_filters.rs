use crate::constants::{CATALOG_PAGE_SIZE, TYPE_PRIORITIES};
use crate::models::common::{compare_with_priorities, eq_update};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::library::{LibraryBucket, LibraryItem};
use boolinator::Boolinator;
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::iter;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
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
    pub r#type: Option<String>,
    #[serde(default)]
    pub sort: Sort,
    #[serde(default = "default_page")]
    pub page: NonZeroUsize,
}

fn default_page() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub request: LibraryRequest,
}

#[derive(PartialEq, Serialize)]
pub struct SelectableType {
    pub r#type: Option<String>,
    pub selected: bool,
    pub request: LibraryRequest,
}

#[derive(PartialEq, Serialize)]
pub struct SelectableSort {
    pub sort: Sort,
    pub selected: bool,
    pub request: LibraryRequest,
}

#[derive(PartialEq, Serialize)]
pub struct SelectablePage {
    pub request: LibraryRequest,
}

#[derive(Default, PartialEq, Serialize)]
pub struct Selectable {
    pub types: Vec<SelectableType>,
    pub sorts: Vec<SelectableSort>,
    pub prev_page: Option<SelectablePage>,
    pub next_page: Option<SelectablePage>,
}

#[derive(Derivative, Serialize)]
#[derivative(Default(bound = ""))]
pub struct LibraryWithFilters<F> {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalog: Vec<LibraryItem>,
    #[serde(skip)]
    pub filter: PhantomData<F>,
}

impl<F: LibraryFilter> LibraryWithFilters<F> {
    pub fn new(library: &LibraryBucket) -> (Self, Effects) {
        let selected = None;
        let mut selectable = Selectable::default();
        let effects = selectable_update::<F>(&mut selectable, &selected, &library);
        (
            Self {
                selectable,
                selected,
                ..Self::default()
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
                let catalog_effects =
                    catalog_update::<F>(&mut self.catalog, &self.selected, &ctx.library);
                selected_effects
                    .join(selectable_effects)
                    .join(catalog_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let selectable_effects =
                    selectable_update::<F>(&mut self.selectable, &self.selected, &ctx.library);
                let catalog_effects =
                    catalog_update::<F>(&mut self.catalog, &self.selected, &ctx.library);
                selected_effects
                    .join(selectable_effects)
                    .join(catalog_effects)
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                let selectable_effects =
                    selectable_update::<F>(&mut self.selectable, &self.selected, &ctx.library);
                let catalog_effects =
                    catalog_update::<F>(&mut self.catalog, &self.selected, &ctx.library);
                selectable_effects.join(catalog_effects)
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
        .map(|library_item| &library_item.r#type)
        .unique()
        .sorted_by(|a, b| compare_with_priorities(a.as_str(), b.as_str(), &*TYPE_PRIORITIES))
        .rev()
        .cloned()
        .map(Some)
        .map(|r#type| SelectableType {
            r#type: r#type.to_owned(),
            request: LibraryRequest {
                r#type: r#type.to_owned(),
                sort: selected
                    .as_ref()
                    .map(|selected| selected.request.sort.to_owned())
                    .unwrap_or_default(),
                page: NonZeroUsize::new(1).unwrap(),
            },
            selected: selected
                .as_ref()
                .map(|selected| selected.request.r#type == r#type)
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    let selectable_types = iter::once(SelectableType {
        r#type: None,
        request: LibraryRequest {
            r#type: None,
            sort: selected
                .as_ref()
                .map(|selected| selected.request.sort.to_owned())
                .unwrap_or_default(),
            page: NonZeroUsize::new(1).unwrap(),
        },
        selected: selected
            .as_ref()
            .map(|selected| selected.request.r#type.is_none())
            .unwrap_or_default(),
    })
    .chain(selectable_types.into_iter())
    .collect::<Vec<_>>();
    let selectable_sorts = Sort::iter()
        .map(|sort| SelectableSort {
            sort: sort.to_owned(),
            request: LibraryRequest {
                r#type: selected
                    .as_ref()
                    .and_then(|selected| selected.request.r#type.to_owned()),
                sort: sort.to_owned(),
                page: NonZeroUsize::new(1).unwrap(),
            },
            selected: selected
                .as_ref()
                .map(|selected| selected.request.sort == sort)
                .unwrap_or_default(),
        })
        .collect();
    let (prev_page, next_page) = match selected {
        Some(selected) => {
            let prev_page = (selected.request.page.get() > 1)
                .as_option()
                .map(|_| SelectablePage {
                    request: LibraryRequest {
                        page: NonZeroUsize::new(selected.request.page.get() - 1).unwrap(),
                        ..selected.request.to_owned()
                    },
                });
            let next_page = library
                .items
                .values()
                .filter(|library_item| F::predicate(library_item))
                .filter(|library_item| match &selected.request.r#type {
                    Some(r#type) => library_item.r#type == *r#type,
                    None => true,
                })
                .nth(selected.request.page.get() * CATALOG_PAGE_SIZE)
                .map(|_| SelectablePage {
                    request: LibraryRequest {
                        page: NonZeroUsize::new(selected.request.page.get() + 1).unwrap(),
                        ..selected.request.to_owned()
                    },
                });
            (prev_page, next_page)
        }
        _ => Default::default(),
    };
    let next_selectable = Selectable {
        types: selectable_types,
        sorts: selectable_sorts,
        prev_page,
        next_page,
    };
    eq_update(selectable, next_selectable)
}

fn catalog_update<F: LibraryFilter>(
    catalog: &mut Vec<LibraryItem>,
    selected: &Option<Selected>,
    library: &LibraryBucket,
) -> Effects {
    let next_catalog = match selected {
        Some(selected) => library
            .items
            .values()
            .filter(|library_item| F::predicate(library_item))
            .filter(|library_item| match &selected.request.r#type {
                Some(r#type) => library_item.r#type == *r#type,
                None => true,
            })
            .sorted_by(|a, b| match &selected.request.sort {
                Sort::LastWatched => b.state.last_watched.cmp(&a.state.last_watched),
                Sort::TimesWatched => b.state.times_watched.cmp(&a.state.times_watched),
                Sort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            })
            .skip((selected.request.page.get() - 1) * CATALOG_PAGE_SIZE)
            .take(CATALOG_PAGE_SIZE)
            .cloned()
            .collect(),
        _ => vec![],
    };
    eq_update(catalog, next_catalog)
}
