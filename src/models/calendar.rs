use chrono::{DateTime, Datelike, Months, NaiveDate, Utc};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    constants::{CALENDAR_IDS_EXTRA_PROP, CALENDAR_ITEMS_COUNT},
    runtime::{
        msg::{Action, ActionLoad, Internal, Msg},
        Effects, Env, UpdateWithCtx,
    },
    types::{
        addon::{AggrRequest, Descriptor, ExtraType},
        library::LibraryBucket,
        resource::{MetaItem, Video},
    },
};

use crate::models::{
    common::{
        eq_update, resources_update_with_vector_content, Loadable, ResourceLoadable,
        ResourcesAction,
    },
    ctx::Ctx,
};

pub type Day = u32;
pub type Month = u32;
pub type Year = i32;

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct YearMonthDate {
    pub month: Month,
    pub year: Year,
}

impl From<NaiveDate> for YearMonthDate {
    fn from(value: NaiveDate) -> Self {
        Self {
            month: value.month(),
            year: value.year(),
        }
    }
}

impl From<DateTime<Utc>> for YearMonthDate {
    fn from(value: DateTime<Utc>) -> Self {
        Self {
            month: value.month(),
            year: value.year(),
        }
    }
}

impl From<Option<DateTime<Utc>>> for YearMonthDate {
    fn from(value: Option<DateTime<Utc>>) -> Self {
        match value {
            Some(date) => Self::from(date),
            None => Self::default(),
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct FullDate {
    pub day: Day,
    pub month: Month,
    pub year: Year,
}

impl FullDate {
    pub fn new(year: Year, month: Month, day: Day) -> Self {
        Self { day, month, year }
    }
}

pub type Selected = YearMonthDate;

#[derive(Default, Clone, PartialEq, Eq, Serialize, Debug)]
pub struct Selectable {
    pub prev: YearMonthDate,
    pub next: YearMonthDate,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentItem {
    pub meta_item: MetaItem,
    pub video: Video,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
pub struct Item {
    pub date: FullDate,
    pub items: Vec<ContentItem>,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MonthInfo {
    pub today: Option<Day>,
    pub days: u32,
    pub first_weekday: u32,
}

#[derive(Derivative, Clone, Serialize, Debug)]
#[derivative(Default(bound = ""))]
pub struct Calendar {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub meta_items: Vec<ResourceLoadable<Vec<MetaItem>>>,
    pub month_info: MonthInfo,
    pub items: Vec<Item>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for Calendar {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Calendar(selected))) => {
                let meta_items_effects =
                    meta_items_update::<E>(&mut self.meta_items, &ctx.library, &ctx.profile.addons);
                let selected_effects = selected_update::<E>(&mut self.selected, selected);
                let month_info_effects =
                    month_info_update::<E>(&mut self.month_info, &self.selected);
                let selectable_effects = selectable_update(&mut self.selectable, &self.selected);
                let items_effects = items_update(
                    &mut self.items,
                    &self.selected,
                    &self.month_info,
                    &self.meta_items,
                );

                meta_items_effects
                    .join(selected_effects)
                    .join(month_info_effects)
                    .join(selectable_effects)
                    .join(items_effects)
            }
            Msg::Action(Action::Unload) => {
                let meta_items_effects = eq_update(&mut self.meta_items, Vec::new());
                let selected_effects = eq_update(&mut self.selected, None);
                let month_info_effects = eq_update(&mut self.month_info, MonthInfo::default());
                let selectable_effects = eq_update(&mut self.selectable, Selectable::default());
                let items_effects = eq_update(&mut self.items, Vec::new());

                meta_items_effects
                    .join(selected_effects)
                    .join(month_info_effects)
                    .join(selectable_effects)
                    .join(items_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                let meta_items_effects = resources_update_with_vector_content::<E, _>(
                    &mut self.meta_items,
                    ResourcesAction::ResourceRequestResult { request, result },
                );

                let items_effects = items_update(
                    &mut self.items,
                    &self.selected,
                    &self.month_info,
                    &self.meta_items,
                );

                meta_items_effects.join(items_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn selected_update<E: Env + 'static>(
    selected: &mut Option<Selected>,
    next_selected: &Option<Selected>,
) -> Effects {
    let current_date = E::now();

    let updated_selected = next_selected
        .as_ref()
        .map(|next_selected| next_selected.to_owned())
        .or(Some(YearMonthDate::from(current_date)));

    eq_update(selected, updated_selected)
}

fn month_info_update<E: Env + 'static>(
    month_info: &mut MonthInfo,
    selected: &Option<Selected>,
) -> Effects {
    let updated_month_info = selected
        .as_ref()
        .map(|Selected { month, year, .. }| {
            let current_date = E::now();

            let today = if current_date.year() == *year && current_date.month() == *month {
                Some(current_date.day())
            } else {
                None
            };

            let first_day_of_month = NaiveDate::from_ymd_opt(*year, *month, 1).unwrap_or_default();
            let first_of_next_month = first_day_of_month + Months::new(1);
            let last_day_of_month = first_of_next_month.pred_opt().unwrap_or_default();

            let days = last_day_of_month.day();

            let first_weekday = first_day_of_month.weekday().num_days_from_monday();

            MonthInfo {
                today,
                days,
                first_weekday,
            }
        })
        .unwrap_or_default();

    eq_update(month_info, updated_month_info)
}

fn selectable_update(selectable: &mut Selectable, selected: &Option<Selected>) -> Effects {
    let updated_selectable = selected
        .as_ref()
        .map(|Selected { month, year, .. }| {
            let date = NaiveDate::from_ymd_opt(*year, *month, 1).unwrap_or_default();

            let prev_date = date - Months::new(1);
            let next_date = date + Months::new(1);

            Selectable {
                prev: YearMonthDate::from(prev_date),
                next: YearMonthDate::from(next_date),
            }
        })
        .unwrap_or_default();

    eq_update(selectable, updated_selectable)
}

fn items_update(
    items: &mut Vec<Item>,
    selected: &Option<Selected>,
    month_info: &MonthInfo,
    meta_items: &[ResourceLoadable<Vec<MetaItem>>],
) -> Effects {
    let updated_items = selected
        .as_ref()
        .map(|Selected { month, year, .. }| {
            (1..=month_info.days)
                .map(|day| Item {
                    date: FullDate::new(*year, *month, day),
                    items: meta_items
                        .iter()
                        .flat_map(|ResourceLoadable { content, .. }| match content {
                            Some(Loadable::Ready(content)) => content
                                .iter()
                                .flat_map(|meta_item| {
                                    meta_item
                                        .videos
                                        .iter()
                                        .filter(|video| {
                                            video
                                                .released
                                                .map(|released| {
                                                    released.day() == day
                                                        && released.month() == *month
                                                        && released.year() == *year
                                                })
                                                .unwrap_or(false)
                                        })
                                        .map(|video| ContentItem {
                                            meta_item: meta_item.clone(),
                                            video: video.clone(),
                                        })
                                        .collect_vec()
                                })
                                .collect_vec(),
                            _ => vec![],
                        })
                        .collect_vec(),
                })
                .collect_vec()
        })
        .unwrap_or_default();

    eq_update(items, updated_items)
}

fn meta_items_update<E: Env + 'static>(
    meta_items: &mut Vec<ResourceLoadable<Vec<MetaItem>>>,
    library: &LibraryBucket,
    addons: &[Descriptor],
) -> Effects {
    if meta_items.is_empty() {
        let id_types = library
            .items
            .values()
            .filter(|library_item| !library_item.removed && !library_item.temp)
            .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
            .map(|library_item| (library_item.id.to_owned(), library_item.r#type.to_owned()))
            .collect_vec();

        resources_update_with_vector_content::<E, _>(
            meta_items,
            ResourcesAction::force_request(
                &AggrRequest::CatalogsFiltered(vec![ExtraType::Ids {
                    extra_name: CALENDAR_IDS_EXTRA_PROP.name.to_owned(),
                    id_types,
                    limit: Some(CALENDAR_ITEMS_COUNT),
                }]),
                addons,
            ),
        )
    } else {
        Effects::none().unchanged()
    }
}
