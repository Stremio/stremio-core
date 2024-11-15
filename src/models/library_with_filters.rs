use std::{cmp::Ordering, iter, marker::PhantomData, num::NonZeroUsize};

use derivative::Derivative;
use derive_more::Deref;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    constants::{CATALOG_PAGE_SIZE, TYPE_PRIORITIES},
    models::{
        common::{compare_with_priorities, eq_update},
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionLibraryWithFilters, ActionLoad, Internal, Msg},
        Effects, Env, UpdateWithCtx,
    },
    types::{
        library::{LibraryBucket, LibraryItem},
        notifications::NotificationsBucket,
    },
};

pub trait LibraryFilter {
    fn predicate(library_item: &LibraryItem, notifications: &NotificationsBucket) -> bool;
}

#[derive(Clone, Debug)]
pub enum ContinueWatchingFilter {}

impl LibraryFilter for ContinueWatchingFilter {
    fn predicate(library_item: &LibraryItem, notifications: &NotificationsBucket) -> bool {
        let library_notification = notifications
            .items
            .get(&library_item.id)
            .filter(|meta_notifs| !meta_notifs.is_empty());

        library_item.is_in_continue_watching() || library_notification.is_some()
    }
}

#[derive(Clone, Debug)]
pub enum NotRemovedFilter {}

impl LibraryFilter for NotRemovedFilter {
    fn predicate(library_item: &LibraryItem, _notifications: &NotificationsBucket) -> bool {
        !library_item.removed
    }
}

#[derive(Derivative, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize, Debug)]
#[derivative(Default)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    #[derivative(Default)]
    LastWatched,
    Name,
    NameReverse,
    TimesWatched,
    Watched,
    NotWatched,
}

impl Sort {
    /// [`Sort`]ing the two given [`LibraryItem`]s for the Library
    pub fn sort_items(&self, a: &LibraryItem, b: &LibraryItem) -> Ordering {
        match &self {
            Sort::LastWatched => b.state.last_watched.cmp(&a.state.last_watched),
            Sort::TimesWatched => b.state.times_watched.cmp(&a.state.times_watched),
            // the only difference between the Watched and Not watched sorting
            // is the ordering of the `a` and `b` items
            Sort::Watched => b
                .watched()
                .cmp(&a.watched())
                .then(b.state.last_watched.cmp(&a.state.last_watched))
                // only as fallback
                // when a new item is added to the library, `last_watched` is always set to now
                // same as `ctime`
                .then(b.ctime.cmp(&a.ctime)),
            Sort::NotWatched => a
                .watched()
                .cmp(&b.watched())
                .then(a.state.last_watched.cmp(&b.state.last_watched))
                // only as fallback
                // when a new item is added to the library, `last_watched` is always set to now
                // same as `ctime`
                .then(a.ctime.cmp(&b.ctime)),
            Sort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            Sort::NameReverse => b.name.to_lowercase().cmp(&a.name.to_lowercase()),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LibraryRequest {
    pub r#type: Option<String>,
    #[serde(default)]
    pub sort: Sort,
    #[serde(default)]
    pub page: LibraryRequestPage,
}

#[derive(Clone, Deref, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LibraryRequestPage(pub NonZeroUsize);

impl Default for LibraryRequestPage {
    fn default() -> LibraryRequestPage {
        LibraryRequestPage(NonZeroUsize::new(1).unwrap())
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Selected {
    pub request: LibraryRequest,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
pub struct SelectableType {
    pub r#type: Option<String>,
    pub selected: bool,
    pub request: LibraryRequest,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
pub struct SelectableSort {
    pub sort: Sort,
    pub selected: bool,
    pub request: LibraryRequest,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
pub struct SelectablePage {
    pub request: LibraryRequest,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Debug)]
pub struct Selectable {
    pub types: Vec<SelectableType>,
    pub sorts: Vec<SelectableSort>,
    pub next_page: Option<SelectablePage>,
}

#[derive(Derivative, Clone, Serialize, Debug)]
#[derivative(Default(bound = ""))]
pub struct LibraryWithFilters<F> {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalog: Vec<LibraryItem>,
    #[serde(skip)]
    pub filter: PhantomData<F>,
}

impl<F: LibraryFilter> LibraryWithFilters<F> {
    pub fn new(library: &LibraryBucket, notifications: &NotificationsBucket) -> (Self, Effects) {
        let selected = None;
        let mut selectable = Selectable::default();
        let effects = selectable_update::<F>(&mut selectable, &selected, library, notifications);
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

impl<E: Env + 'static, F: LibraryFilter> UpdateWithCtx<E> for LibraryWithFilters<F> {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryWithFilters(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let selectable_effects = selectable_update::<F>(
                    &mut self.selectable,
                    &self.selected,
                    &ctx.library,
                    &ctx.notifications,
                );
                let catalog_effects = catalog_update::<F>(
                    &mut self.catalog,
                    &self.selected,
                    &ctx.library,
                    &ctx.notifications,
                );
                selected_effects
                    .join(selectable_effects)
                    .join(catalog_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let selectable_effects = selectable_update::<F>(
                    &mut self.selectable,
                    &self.selected,
                    &ctx.library,
                    &ctx.notifications,
                );
                let catalog_effects = catalog_update::<F>(
                    &mut self.catalog,
                    &self.selected,
                    &ctx.library,
                    &ctx.notifications,
                );
                selected_effects
                    .join(selectable_effects)
                    .join(catalog_effects)
            }
            Msg::Action(Action::LibraryWithFilters(ActionLibraryWithFilters::LoadNextPage)) => {
                match self.selectable.next_page.as_ref() {
                    Some(next_page) => {
                        let next_selected = Some(Selected {
                            request: next_page.request.to_owned(),
                        });
                        let selected_effects = eq_update(&mut self.selected, next_selected);
                        let selectable_effects = selectable_update::<F>(
                            &mut self.selectable,
                            &self.selected,
                            &ctx.library,
                            &ctx.notifications,
                        );
                        let catalog_effects = catalog_update::<F>(
                            &mut self.catalog,
                            &self.selected,
                            &ctx.library,
                            &ctx.notifications,
                        );
                        selected_effects
                            .join(selectable_effects)
                            .join(catalog_effects)
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                let selectable_effects = selectable_update::<F>(
                    &mut self.selectable,
                    &self.selected,
                    &ctx.library,
                    &ctx.notifications,
                );
                let catalog_effects = catalog_update::<F>(
                    &mut self.catalog,
                    &self.selected,
                    &ctx.library,
                    &ctx.notifications,
                );
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
    notifications: &NotificationsBucket,
) -> Effects {
    let selectable_types = library
        .items
        .values()
        .filter(|library_item| F::predicate(library_item, notifications))
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
                page: LibraryRequestPage::default(),
            },
            selected: selected
                .as_ref()
                .map(|selected| selected.request.r#type == r#type)
                .unwrap_or_default(),
        });
    let selectable_types = iter::once(SelectableType {
        r#type: None,
        request: LibraryRequest {
            r#type: None,
            sort: selected
                .as_ref()
                .map(|selected| selected.request.sort.to_owned())
                .unwrap_or_default(),
            page: LibraryRequestPage::default(),
        },
        selected: selected
            .as_ref()
            .map(|selected| selected.request.r#type.is_none())
            .unwrap_or_default(),
    })
    .chain(selectable_types)
    .collect::<Vec<_>>();
    let selectable_sorts = Sort::iter()
        .map(|sort| SelectableSort {
            sort: sort.to_owned(),
            request: LibraryRequest {
                r#type: selected
                    .as_ref()
                    .and_then(|selected| selected.request.r#type.to_owned()),
                sort: sort.to_owned(),
                page: LibraryRequestPage::default(),
            },
            selected: selected
                .as_ref()
                .map(|selected| selected.request.sort == sort)
                .unwrap_or_default(),
        })
        .collect();
    let next_page = match selected {
        Some(selected) => library
            .items
            .values()
            .filter(|library_item| F::predicate(library_item, notifications))
            .filter(|library_item| match &selected.request.r#type {
                Some(r#type) => library_item.r#type == *r#type,
                None => true,
            })
            .nth(selected.request.page.get() * CATALOG_PAGE_SIZE)
            .map(|_| SelectablePage {
                request: LibraryRequest {
                    page: LibraryRequestPage(
                        NonZeroUsize::new(selected.request.page.get() + 1).unwrap(),
                    ),
                    ..selected.request.to_owned()
                },
            }),
        _ => Default::default(),
    };
    let next_selectable = Selectable {
        types: selectable_types,
        sorts: selectable_sorts,
        next_page,
    };
    eq_update(selectable, next_selectable)
}

fn catalog_update<F: LibraryFilter>(
    catalog: &mut Vec<LibraryItem>,
    selected: &Option<Selected>,
    library: &LibraryBucket,
    notifications: &NotificationsBucket,
) -> Effects {
    let next_catalog = match selected {
        Some(selected) => library
            .items
            .values()
            .filter(|library_item| F::predicate(library_item, notifications))
            .filter(|library_item| match &selected.request.r#type {
                Some(r#type) => library_item.r#type == *r#type,
                None => true,
            })
            .sorted_by(|a, b| selected.request.sort.sort_items(a, b))
            .take(selected.request.page.get() * CATALOG_PAGE_SIZE)
            .cloned()
            .collect(),
        _ => vec![],
    };
    eq_update(catalog, next_catalog)
}

#[cfg(test)]
mod test {
    use chrono::{Duration, Utc};

    use crate::types::{
        library::{LibraryItem, LibraryItemState},
        resource::PosterShape,
    };

    use super::Sort;

    #[test]
    fn test_watched_and_not_watched_sort_items_ordering_of_library_items() {
        // For series, times_watched is incremented to indicate that a single or more
        // episodes have been watched
        // While last_watched is used to order the watched items
        // And flagged_watched is not used to show the watched indicator
        let watched_latest_series = LibraryItem {
            id: "tt13622776".into(),
            name: "Ahsoka".into(),
            r#type: "series".into(),
            poster: None,
            poster_shape: PosterShape::Poster,
            removed: false,
            temp: false,
            ctime: Some(Utc::now()),
            mtime: Utc::now(),
            state: LibraryItemState {
                last_watched: Some(Utc::now()),
                // Series has been watched
                flagged_watched: 1,
                // indicate 2 watched videos
                times_watched: 2,
                ..Default::default()
            },
            behavior_hints: crate::types::resource::MetaItemBehaviorHints::default(),
        };
        let watched_movie_1_week_ago = LibraryItem {
            id: "tt15398776".into(),
            name: "Oppenheimer".into(),
            r#type: "movie".into(),
            poster: None,
            poster_shape: PosterShape::Poster,
            removed: false,
            temp: false,
            ctime: Some(Utc::now()),
            mtime: Utc::now(),
            state: LibraryItemState {
                last_watched: Some(Utc::now() - Duration::weeks(1)),
                flagged_watched: 1,
                times_watched: 1,
                ..Default::default()
            },
            behavior_hints: crate::types::resource::MetaItemBehaviorHints::default(),
        };

        let not_watched_movie_added_3_weeks_ago = LibraryItem {
            id: "tt2267998".into(),
            name: "Gone Girl".into(),
            r#type: "movie".into(),
            poster: None,
            poster_shape: PosterShape::Poster,
            removed: false,
            temp: false,
            ctime: Some(Utc::now() - Duration::weeks(3)),
            mtime: Utc::now(),
            state: LibraryItemState {
                last_watched: Some(Utc::now() - Duration::weeks(3)),
                flagged_watched: 0,
                times_watched: 0,
                ..Default::default()
            },
            behavior_hints: crate::types::resource::MetaItemBehaviorHints::default(),
        };

        let not_watched_movie_added_2_weeks_ago = LibraryItem {
            id: "tt0118715".into(),
            name: "The Big Lebowski".into(),
            r#type: "movie".into(),
            poster: None,
            poster_shape: PosterShape::Poster,
            removed: false,
            temp: false,
            ctime: Some(Utc::now() - Duration::weeks(2)),
            mtime: Utc::now(),
            state: LibraryItemState {
                last_watched: Some(Utc::now() - Duration::weeks(2)),
                flagged_watched: 0,
                times_watched: 0,
                ..Default::default()
            },
            behavior_hints: crate::types::resource::MetaItemBehaviorHints::default(),
        };

        let watched_movie_1_week_ago_marked_not_watched = LibraryItem {
            id: "tt1462764".into(),
            name: "Indiana Jones and the Dial of Destiny".into(),
            r#type: "movie".into(),
            poster: None,
            poster_shape: PosterShape::Poster,
            removed: false,
            temp: false,
            ctime: Some(Utc::now() - Duration::weeks(4)),
            mtime: Utc::now(),
            state: LibraryItemState {
                last_watched: Some(Utc::now() - Duration::weeks(1)),
                flagged_watched: 0,
                times_watched: 0,
                ..Default::default()
            },
            behavior_hints: crate::types::resource::MetaItemBehaviorHints::default(),
        };

        // Sort by Watched - first library items that are Watched by latest `last_watched` desc
        // and then not watched and creation time (`ctime`) desc
        {
            let mut items = vec![
                &not_watched_movie_added_3_weeks_ago,
                &watched_movie_1_week_ago_marked_not_watched,
                &not_watched_movie_added_2_weeks_ago,
                &watched_movie_1_week_ago,
                &watched_latest_series,
            ];

            items.sort_by(|a, b| Sort::Watched.sort_items(a, b));

            pretty_assertions::assert_eq!(
                items,
                vec![
                    &watched_latest_series,
                    &watched_movie_1_week_ago,
                    &watched_movie_1_week_ago_marked_not_watched,
                    &not_watched_movie_added_2_weeks_ago,
                    &not_watched_movie_added_3_weeks_ago,
                ]
            )
        }

        {
            let mut items = vec![
                &not_watched_movie_added_3_weeks_ago,
                &watched_latest_series,
                &not_watched_movie_added_2_weeks_ago,
                &watched_movie_1_week_ago,
                &watched_movie_1_week_ago_marked_not_watched,
            ];

            items.sort_by(|a, b| Sort::NotWatched.sort_items(a, b));

            pretty_assertions::assert_eq!(
                items,
                vec![
                    &not_watched_movie_added_3_weeks_ago,
                    &not_watched_movie_added_2_weeks_ago,
                    &watched_movie_1_week_ago_marked_not_watched,
                    &watched_movie_1_week_ago,
                    &watched_latest_series,
                ]
            )
        }
    }
}
