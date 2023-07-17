#[cfg(debug_assertions)]
use serde::Serialize;

use stremio_core::models::addon_details::AddonDetails;
use stremio_core::models::catalog_with_filters::CatalogWithFilters;
use stremio_core::models::catalogs_with_extra::CatalogsWithExtra;
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::models::data_export::DataExport;
use stremio_core::models::installed_addons_with_filters::InstalledAddonsWithFilters;
use stremio_core::models::library_with_filters::{
    ContinueWatchingFilter, LibraryWithFilters, NotRemovedFilter,
};
use stremio_core::models::link::Link;
use stremio_core::models::meta_details::MetaDetails;
use stremio_core::models::player::Player;
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::runtime::Effects;
use stremio_core::types::addon::DescriptorPreview;
use stremio_core::types::api::LinkAuthKey;
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::profile::Profile;
use stremio_core::types::resource::MetaItemPreview;
use stremio_core::types::streams::StreamsBucket;
use stremio_derive::Model;

use wasm_bindgen::JsValue;

use crate::env::WebEnv;
use crate::model::{
    serialize_catalogs_with_extra, serialize_continue_watching_preview, serialize_data_export,
    serialize_discover, serialize_installed_addons, serialize_library, serialize_meta_details,
    serialize_player, serialize_remote_addons, serialize_streaming_server,
};

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
    pub search: CatalogsWithExtra,
    pub meta_details: MetaDetails,
    pub remote_addons: CatalogWithFilters<DescriptorPreview>,
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
    ) -> (WebModel, Effects) {
        let (continue_watching_preview, continue_watching_preview_effects) =
            ContinueWatchingPreview::new(&library);
        let (discover, discover_effects) = CatalogWithFilters::<MetaItemPreview>::new(&profile);
        let (library_, library_effects) = LibraryWithFilters::<NotRemovedFilter>::new(&library);
        let (continue_watching, continue_watching_effects) =
            LibraryWithFilters::<ContinueWatchingFilter>::new(&library);
        let (remote_addons, remote_addons_effects) =
            CatalogWithFilters::<DescriptorPreview>::new(&profile);
        let (installed_addons, installed_addons_effects) =
            InstalledAddonsWithFilters::new(&profile);
        let (streaming_server, streaming_server_effects) = StreamingServer::new::<WebEnv>(&profile);
        let model = WebModel {
            ctx: Ctx::new(profile, library, streams),
            auth_link: Default::default(),
            data_export: Default::default(),
            continue_watching_preview,
            board: Default::default(),
            discover,
            library: library_,
            continue_watching,
            search: Default::default(),
            meta_details: Default::default(),
            remote_addons,
            installed_addons,
            addon_details: Default::default(),
            streaming_server,
            player: Default::default(),
        };
        (
            model,
            continue_watching_preview_effects
                .join(discover_effects)
                .join(library_effects)
                .join(continue_watching_effects)
                .join(remote_addons_effects)
                .join(installed_addons_effects)
                .join(streaming_server_effects),
        )
    }
    pub fn get_state(&self, field: &WebModelField) -> JsValue {
        match field {
            WebModelField::Ctx => JsValue::from_serde(&self.ctx).unwrap(),
            WebModelField::AuthLink => JsValue::from_serde(&self.auth_link).unwrap(),
            WebModelField::DataExport => serialize_data_export(&self.data_export),
            WebModelField::ContinueWatchingPreview => {
                serialize_continue_watching_preview(&self.continue_watching_preview)
            }
            WebModelField::Board => serialize_catalogs_with_extra(&self.board, &self.ctx),
            WebModelField::Discover => {
                serialize_discover(&self.discover, &self.ctx, &self.streaming_server)
            }
            WebModelField::Library => serialize_library(&self.library, "library".to_owned()),
            WebModelField::ContinueWatching => {
                serialize_library(&self.continue_watching, "continuewatching".to_owned())
            }
            WebModelField::Search => serialize_catalogs_with_extra(&self.search, &self.ctx),
            WebModelField::MetaDetails => {
                serialize_meta_details(&self.meta_details, &self.ctx, &self.streaming_server)
            }
            WebModelField::RemoteAddons => serialize_remote_addons(&self.remote_addons, &self.ctx),
            WebModelField::InstalledAddons => serialize_installed_addons(&self.installed_addons),
            WebModelField::AddonDetails => JsValue::from_serde(&self.addon_details).unwrap(),
            WebModelField::StreamingServer => serialize_streaming_server(&self.streaming_server),
            WebModelField::Player => {
                serialize_player(&self.player, &self.ctx, &self.streaming_server)
            }
        }
    }
}
