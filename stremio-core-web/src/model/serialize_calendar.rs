use gloo_utils::format::JsValueSerdeExt;
use itertools::Itertools;
use serde::Serialize;
use stremio_core::{
    deep_links::{CalendarDeepLinks, CalendarItemDeepLinks},
    models::calendar::{FullDate, MonthInfo, Selected, YearMonthDate},
    types::resource::SeriesInfo,
};
use url::Url;
use wasm_bindgen::JsValue;

use crate::model::DeepLinksExt;

mod model {
    use super::*;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CalendarContentItem<'a> {
        pub id: &'a String,
        pub name: &'a String,
        pub poster: &'a Option<Url>,
        pub title: &'a String,
        #[serde(flatten)]
        pub series_info: &'a Option<SeriesInfo>,
        pub deep_links: CalendarItemDeepLinks,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CalendarItem<'a> {
        pub date: &'a FullDate,
        pub items: Vec<CalendarContentItem<'a>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableDate<'a> {
        #[serde(flatten)]
        pub date: &'a YearMonthDate,
        pub deep_links: CalendarDeepLinks,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Selectable<'a> {
        pub prev: SelectableDate<'a>,
        pub next: SelectableDate<'a>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Calendar<'a> {
        pub selected: &'a Option<Selected>,
        pub selectable: Selectable<'a>,
        pub month_info: &'a MonthInfo,
        pub items: &'a Vec<CalendarItem<'a>>,
    }
}

#[cfg(feature = "wasm")]
pub fn serialize_calendar(calendar: &stremio_core::models::calendar::Calendar) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::Calendar {
        selected: &calendar.selected,
        selectable: model::Selectable {
            prev: model::SelectableDate {
                date: &calendar.selectable.prev,
                deep_links: CalendarDeepLinks::from(&calendar.selectable.prev)
                    .into_web_deep_links(),
            },
            next: model::SelectableDate {
                date: &calendar.selectable.next,
                deep_links: CalendarDeepLinks::from(&calendar.selectable.next)
                    .into_web_deep_links(),
            },
        },
        month_info: &calendar.month_info,
        items: &calendar
            .items
            .iter()
            .map(|item| model::CalendarItem {
                date: &item.date,
                items: item
                    .items
                    .iter()
                    .map(|item| model::CalendarContentItem {
                        id: &item.video.id,
                        name: &item.meta_item.preview.name,
                        poster: &item.meta_item.preview.poster,
                        title: &item.video.title,
                        series_info: &item.video.series_info,
                        deep_links: CalendarItemDeepLinks::from((&item.meta_item, &item.video))
                            .into_web_deep_links(),
                    })
                    .unique_by(|item| item.id)
                    .collect_vec(),
            })
            .collect_vec(),
    })
    .expect("JsValue from model::Calendar")
}
