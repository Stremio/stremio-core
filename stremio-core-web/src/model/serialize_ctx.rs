#[cfg(feature = "wasm")]
use {gloo_utils::format::JsValueSerdeExt, wasm_bindgen::JsValue};

pub use model::*;

#[cfg(feature = "wasm")]
pub fn serialize_ctx(ctx: &stremio_core::models::ctx::Ctx) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::Ctx::from(ctx)).expect("JsValue from Ctx")
}

mod model {
    use std::collections::HashMap;

    use chrono::{DateTime, Utc};
    use itertools::Itertools;
    use serde::Serialize;

    use stremio_core::deep_links::SearchHistoryItemDeepLinks;
    use stremio_core::models::catalogs_with_extra::{board_catalogs, board_spotlight_request};
    use stremio_core::types::{
        addon::ResourceRequest, events::Events, notifications::NotificationItem,
        resource::MetaItemId,
    };
    use url::Url;

    use crate::{env::WebEnv, model::deep_links_ext::DeepLinksExt};

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Ctx<'a> {
        pub profile: Profile<'a>,
        pub notifications: Notifications<'a>,
        pub search_history: Vec<SearchHistoryItem<'a>>,
        pub events: &'a Events,
        pub streaming_server_urls: Vec<StreamingServerUrlItem>,
        pub board_catalogs: Vec<BoardCatalog<'a>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManifestPreview<'a> {
        pub id: &'a str,
        pub name: &'a str,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescriptorPreview<'a> {
        pub manifest: ManifestPreview<'a>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BoardCatalog<'a> {
        pub id: &'a str,
        pub name: &'a str,
        pub r#type: &'a str,
        pub addon: DescriptorPreview<'a>,
        pub request: ResourceRequest,
        pub selected: bool,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StreamingServerUrlItem {
        pub url: Url,
        pub mtime: DateTime<Utc>,
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

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Profile<'a> {
        #[serde(flatten)]
        profile: &'a stremio_core::types::profile::Profile,
        #[serde(skip_serializing_if = "Option::is_none")]
        auth: Option<Auth<'a>>,
    }

    #[derive(Serialize)]
    pub struct Auth<'a> {
        pub key: stremio_core::types::profile::AuthKey,
        pub user: User<'a>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct User<'a> {
        #[serde(flatten)]
        pub user: &'a stremio_core::types::profile::User,
        pub is_new_user: bool,
    }

    impl<'a> From<&'a stremio_core::models::ctx::Ctx> for Ctx<'a> {
        fn from(ctx: &'a stremio_core::models::ctx::Ctx) -> Self {
            let spotlight_request = board_spotlight_request(&ctx.profile);
            Self {
                profile: Profile {
                    profile: &ctx.profile,
                    auth: ctx.profile.auth.as_ref().map(|auth| Auth {
                        key: auth.key.clone(),
                        user: User {
                            user: &auth.user,
                            is_new_user: auth.user.is_new_user::<WebEnv>(),
                        },
                    }),
                },
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
                streaming_server_urls: ctx
                    .streaming_server_urls
                    .items
                    .iter()
                    .map(|(url, mtime)| StreamingServerUrlItem {
                        url: url.clone(),
                        mtime: *mtime,
                    })
                    .sorted_by(|a, b| Ord::cmp(&a.mtime, &b.mtime))
                    .collect(),
                board_catalogs: board_catalogs(&ctx.profile.addons)
                    .into_iter()
                    .map(|(addon, catalog, request)| BoardCatalog {
                        id: &catalog.id,
                        name: catalog.name.as_deref().unwrap_or(&addon.manifest.name),
                        r#type: &catalog.r#type,
                        addon: DescriptorPreview {
                            manifest: ManifestPreview {
                                id: &addon.manifest.id,
                                name: &addon.manifest.name,
                            },
                        },
                        selected: spotlight_request.as_ref() == Some(&request),
                        request,
                    })
                    .collect(),
            }
        }
    }
}
