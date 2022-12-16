use crate::constants::{CATALOG_PAGE_SIZE, TYPE_PRIORITIES};
use crate::models::common::{compare_with_priorities, eq_update};
use crate::models::ctx::Ctx;
use crate::models::library_with_filters::{LibraryFilter, Sort};
use crate::runtime::msg::{Action, ActionLibraryByType, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::library::{LibraryBucket, LibraryItem};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use strum::IntoEnumIterator;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Selected {
    #[serde(default)]
    pub sort: Sort,
}

#[derive(PartialEq, Eq, Serialize, Debug)]
pub struct SelectableSort {
    pub sort: Sort,
    pub selected: bool,
}

#[derive(Default, PartialEq, Eq, Serialize, Debug)]
pub struct Selectable {
    pub sorts: Vec<SelectableSort>,
}

pub type CatalogPage = Vec<LibraryItem>;

pub type Catalog = Vec<CatalogPage>;

#[derive(Derivative, Serialize, Debug)]
#[derivative(Default(bound = ""))]
pub struct LibraryByType<F> {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalogs: Vec<Catalog>,
    #[serde(skip)]
    pub filter: PhantomData<F>,
}

impl<F: LibraryFilter> LibraryByType<F> {
    pub fn new() -> (Self, Effects) {
        let selected = None;
        let mut selectable = Selectable::default();
        let effects = selectable_update(&mut selectable, &selected);
        (
            Self {
                selected,
                selectable,
                ..Self::default()
            },
            effects.unchanged(),
        )
    }
}

impl<E: Env + 'static, F: LibraryFilter> UpdateWithCtx<E> for LibraryByType<F> {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryByType(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let selectable_effects = selectable_update(&mut self.selectable, &self.selected);
                let catalogs_effects =
                    catalogs_update::<F>(&mut self.catalogs, &self.selected, &ctx.library);
                selected_effects
                    .join(selectable_effects)
                    .join(catalogs_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let selectable_effects = selectable_update(&mut self.selectable, &self.selected);
                let catalogs_effects = eq_update(&mut self.catalogs, vec![]);
                selected_effects
                    .join(selectable_effects)
                    .join(catalogs_effects)
            }
            Msg::Action(Action::LibraryByType(ActionLibraryByType::LoadNextPage(index))) => {
                match self.catalogs.get_mut(*index) {
                    Some(catalog) => match (catalog.first(), catalog.last()) {
                        (Some(first_page), Some(last_page))
                            if !first_page.is_empty() && last_page.len() == CATALOG_PAGE_SIZE =>
                        {
                            let r#type = first_page
                                .first()
                                .map(|library_item| &library_item.r#type)
                                .expect("first page of library catalog is empty");
                            let skip = catalog.iter().fold(0, |result, page| result + page.len());
                            let page = next_page::<F>(r#type, skip, &self.selected, &ctx.library);
                            catalog.push(page);
                            Effects::none()
                        }
                        _ => Effects::none().unchanged(),
                    },
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                catalogs_update::<F>(&mut self.catalogs, &self.selected, &ctx.library)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn selectable_update(selectable: &mut Selectable, selected: &Option<Selected>) -> Effects {
    let selectable_sorts = Sort::iter()
        .map(|sort| SelectableSort {
            sort: sort.to_owned(),
            selected: selected
                .as_ref()
                .map(|selected| selected.sort == sort)
                .unwrap_or_default(),
        })
        .collect();
    let next_selectable = Selectable {
        sorts: selectable_sorts,
    };
    eq_update(selectable, next_selectable)
}

fn catalogs_update<F: LibraryFilter>(
    catalogs: &mut Vec<Catalog>,
    selected: &Option<Selected>,
    library: &LibraryBucket,
) -> Effects {
    let catalogs_size = catalogs.iter().fold(HashMap::new(), |mut result, catalog| {
        let r#type = catalog
            .first()
            .and_then(|first_page| first_page.first())
            .map(|library_item| &library_item.r#type)
            .expect("first page of library catalog is empty");
        let size = catalog.iter().fold(0, |result, page| result + page.len());
        result.insert(r#type, size);
        result
    });
    let next_catalogs = match selected {
        Some(selected) => library
            .items
            .values()
            .filter(|library_item| F::predicate(library_item))
            .fold(HashMap::new(), |mut result, library_item| {
                result
                    .entry(&library_item.r#type)
                    .or_insert_with(Vec::new)
                    .push(library_item.to_owned());
                result
            })
            .into_iter()
            .sorted_by(|(a_type, _), (b_type, _)| {
                compare_with_priorities(a_type.as_str(), b_type.as_str(), &*TYPE_PRIORITIES)
            })
            .rev()
            .map(|(r#type, library_items)| {
                let take = catalogs_size
                    .get(r#type)
                    .map(|catalog_size| {
                        (*catalog_size as f64 / CATALOG_PAGE_SIZE as f64).ceil() as usize
                            * CATALOG_PAGE_SIZE
                    })
                    .unwrap_or(CATALOG_PAGE_SIZE);
                library_items
                    .into_iter()
                    .sorted_by(|a, b| match &selected.sort {
                        Sort::LastWatched => b.state.last_watched.cmp(&a.state.last_watched),
                        Sort::TimesWatched => b.state.times_watched.cmp(&a.state.times_watched),
                        Sort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    })
                    .take(take)
                    .collect::<Vec<_>>()
                    .chunks(CATALOG_PAGE_SIZE)
                    .map(|page| page.into())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };
    eq_update(catalogs, next_catalogs)
}

fn next_page<F: LibraryFilter>(
    r#type: &String,
    skip: usize,
    selected: &Option<Selected>,
    library: &LibraryBucket,
) -> CatalogPage {
    match selected {
        Some(selected) => library
            .items
            .values()
            .filter(|library_item| F::predicate(library_item))
            .filter(|library_item| library_item.r#type == *r#type)
            .sorted_by(|a, b| match &selected.sort {
                Sort::LastWatched => b.state.last_watched.cmp(&a.state.last_watched),
                Sort::TimesWatched => b.state.times_watched.cmp(&a.state.times_watched),
                Sort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            })
            .skip(skip)
            .take(CATALOG_PAGE_SIZE)
            .map(|library_item| (*library_item).to_owned())
            .collect::<Vec<_>>(),
        _ => vec![],
    }
}
