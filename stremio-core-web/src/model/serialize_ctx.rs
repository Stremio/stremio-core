use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::JsValue;

use stremio_core::models::ctx::Ctx;

pub fn serialize_ctx(ctx: &Ctx) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::Ctx::from(ctx)).expect("JsValue from Ctx")
}

mod model {
    use std::collections::HashMap;

    use chrono::{DateTime, Utc};
    use itertools::Itertools;
    use serde::Serialize;

    use stremio_core::deep_links::SearchHistoryItemDeepLinks;
    use stremio_core::types::{
        events::Events, notifications::NotificationItem, profile::Profile, resource::MetaItemId,
    };

    use crate::model::deep_links_ext::DeepLinksExt;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Ctx<'a> {
        /// keep the original Profile model inside.
        pub profile: &'a Profile,
        pub notifications: Notifications<'a>,
        pub search_history: Vec<SearchHistoryItem<'a>>,
        pub events: &'a Events,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Notifications<'a> {
        /// Override the notifications to simplify the mapping
        pub items: HashMap<MetaItemId, Vec<&'a NotificationItem>>,
        pub last_updated: Option<DateTime<Utc>>,
        pub created: DateTime<Utc>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SearchHistoryItem<'a> {
        pub query: &'a String,
        pub deep_links: SearchHistoryItemDeepLinks,
    }

    impl<'a> From<&'a stremio_core::models::ctx::Ctx> for Ctx<'a> {
        fn from(ctx: &'a stremio_core::models::ctx::Ctx) -> Self {
            Self {
                profile: &ctx.profile,
                notifications: Notifications {
                    items: ctx
                        .notifications
                        .items
                        .iter()
                        .map(|(meta_id, notifications)| {
                            (meta_id.to_owned(), notifications.values().collect())
                        })
                        .collect(),
                    last_updated: ctx.notifications.last_updated,
                    created: ctx.notifications.created,
                },
                search_history: ctx
                    .search_history
                    .items
                    .iter()
                    .sorted_by(|(_, a_date), (_, b_date)| Ord::cmp(b_date, a_date))
                    .map(|(query, ..)| SearchHistoryItem {
                        query,
                        deep_links: SearchHistoryItemDeepLinks::from(query).into_web_deep_links(),
                    })
                    .collect(),
                events: &ctx.events,
            }
        }
    }
}
