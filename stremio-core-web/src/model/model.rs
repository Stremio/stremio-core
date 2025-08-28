use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::JsValue;

use super::*;

#[cfg(debug_assertions)]
use serde::Serialize;

use stremio_core::{
    models::{
        addon_details::AddonDetails,
        calendar::Calendar,
        catalog_with_filters::CatalogWithFilters,
        catalogs_with_extra::CatalogsWithExtra,
        continue_watching_preview::ContinueWatchingPreview,
        ctx::Ctx,
        data_export::DataExport,
        installed_addons_with_filters::InstalledAddonsWithFilters,
        library_with_filters::{ContinueWatchingFilter, LibraryWithFilters, NotRemovedFilter},
        link::Link,
        local_search::LocalSearch,
        meta_details::MetaDetails,
        player::Player,
        streaming_server::StreamingServer,
    },
    runtime::Effects,
    types::{
        addon::Descriptor, api::LinkAuthKey, events::DismissedEventsBucket, library::LibraryBucket,
        notifications::NotificationsBucket, profile::Profile, resource::MetaItemPreview,
        search_history::SearchHistoryBucket, server_urls::ServerUrlsBucket, streams::StreamsBucket,
    },
    Model,
};

use super::SerializeModel;
use crate::env::WebEnv;

#[derive(Model, Clone)]
#[cfg_attr(debug_assertions, derive(Serialize))]
#[model(WebEnv)]
pub struct WebModel {
    pub ctx: Ctx,
    pub auth_link: Link<LinkAuthKey>,
    pub data_export: DataExport,
    pub continue_watching_preview: ContinueWatchingPreview,
    pub board: CatalogsWithExtra,
    pub discover: CatalogWithFilters<MetaItemPreview>,
    pub library: LibraryWithFilters<NotRemovedFilter>,
    pub continue_watching: LibraryWithFilters<ContinueWatchingFilter>,
    pub calendar: Calendar,
    pub search: CatalogsWithExtra,
    /// Pre-loaded results for local search
    pub local_search: LocalSearch,
    pub meta_details: MetaDetails,
    pub remote_addons: CatalogWithFilters<Descriptor>,
    pub installed_addons: InstalledAddonsWithFilters,
    pub addon_details: AddonDetails,
    pub streaming_server: StreamingServer,
    pub player: Player,
}

impl WebModel {
    pub fn new(
        profile: Profile,
        library: LibraryBucket,
        streams: StreamsBucket,
        server_urls: ServerUrlsBucket,
        notifications: NotificationsBucket,
        search_history: SearchHistoryBucket,
        dismissed_events: DismissedEventsBucket,
    ) -> (WebModel, Effects) {
        let (continue_watching_preview, continue_watching_preview_effects) =
            ContinueWatchingPreview::new(&library, &notifications);
        let (discover, discover_effects) = CatalogWithFilters::<MetaItemPreview>::new(&profile);
        let (library_, library_effects) =
            LibraryWithFilters::<NotRemovedFilter>::new(&library, &notifications);
        let (continue_watching, continue_watching_effects) =
            LibraryWithFilters::<ContinueWatchingFilter>::new(&library, &notifications);
        let (remote_addons, remote_addons_effects) =
            CatalogWithFilters::<Descriptor>::new(&profile);
        let (installed_addons, installed_addons_effects) =
            InstalledAddonsWithFilters::new(&profile);
        let (streaming_server, streaming_server_effects) = StreamingServer::new::<WebEnv>(&profile);
        let (local_search, local_search_effects) = LocalSearch::new::<WebEnv>();
        let model = WebModel {
            ctx: Ctx::new(
                profile,
                library,
                streams,
                server_urls,
                notifications,
                search_history,
                dismissed_events,
            ),
            auth_link: Default::default(),
            data_export: Default::default(),
            local_search,
            continue_watching_preview,
            board: Default::default(),
            discover,
            library: library_,
            continue_watching,
            calendar: Default::default(),
            search: Default::default(),
            meta_details: Default::default(),
            remote_addons,
            installed_addons,
            addon_details: Default::default(),
            streaming_server,
            player: Player {
                collect_seek_logs: true,
                ..Default::default()
            },
        };
        (
            model,
            continue_watching_preview_effects
                .join(discover_effects)
                .join(library_effects)
                .join(continue_watching_effects)
                .join(remote_addons_effects)
                .join(installed_addons_effects)
                .join(streaming_server_effects)
                .join(local_search_effects),
        )
    }
    pub fn get_state(&self, field: &WebModelField) -> JsValue {
        match field {
            WebModelField::Ctx => serialize_ctx(&self.ctx),
            WebModelField::AuthLink => <JsValue as JsValueSerdeExt>::from_serde(&self.auth_link)
                .expect("JsValue from AuthLink"),
            WebModelField::DataExport => serialize_data_export(&self.data_export),
            WebModelField::ContinueWatchingPreview => serialize_continue_watching_preview(
                &self.continue_watching_preview,
                &self.ctx.streams,
                self.streaming_server.base_url.as_ref(),
                &self.ctx.profile.settings,
            ),
            WebModelField::Board => {
                // let old = serialize_catalogs_with_extra(&self.board, &self.ctx);
                crate::model::serialize_catalogs_with_extra::CatalogsWithExtra::new(
                    &self.board,
                    &self.ctx,
                    &self.streaming_server,
                )
                .serialize_model()
                .expect("JsValue from model::CatalogsWithExtra")
            }
            WebModelField::Discover => {
                serialize_discover(&self.discover, &self.ctx, &self.streaming_server)
            }
            WebModelField::Library => serialize_library(
                &self.library,
                &self.ctx,
                self.streaming_server.base_url.as_ref(),
                "library".to_owned(),
            ),
            WebModelField::ContinueWatching => serialize_library(
                &self.continue_watching,
                &self.ctx,
                self.streaming_server.base_url.as_ref(),
                "continuewatching".to_owned(),
            ),
            WebModelField::Search => {
                // let old = serialize_catalogs_with_extra(&self.search, &self.ctx)
                crate::model::serialize_catalogs_with_extra::CatalogsWithExtra::new(
                    &self.search,
                    &self.ctx,
                    &self.streaming_server,
                )
                .serialize_model()
                .expect("JsValue from model::CatalogsWithExtra")
            }
            WebModelField::Calendar => serialize_calendar(&self.calendar),
            WebModelField::LocalSearch => serialize_local_search(&self.local_search),
            WebModelField::MetaDetails => serialize_meta_details::<WebEnv>(
                &self.meta_details,
                &self.ctx,
                &self.streaming_server,
            ),
            WebModelField::RemoteAddons => serialize_remote_addons(&self.remote_addons, &self.ctx),
            WebModelField::InstalledAddons => serialize_installed_addons(&self.installed_addons),
            WebModelField::AddonDetails => {
                <JsValue as JsValueSerdeExt>::from_serde(&self.addon_details)
                    .expect("JsValue from AddonDetails")
            }
            WebModelField::StreamingServer => serialize_streaming_server(&self.streaming_server),
            WebModelField::Player => {
                serialize_player::<WebEnv>(&self.player, &self.ctx, &self.streaming_server)
            }
        }
    }
}
