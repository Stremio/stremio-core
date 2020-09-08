use crate::env::Env;
use serde::Serialize;
use stremio_core::models::addon_details::AddonDetails;
use stremio_core::models::catalog_with_filters::CatalogWithFilters;
use stremio_core::models::catalogs_with_extra::CatalogsWithExtra;
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::models::library_with_filters::{
    ContinueWatchingFilter, LibraryWithFilters, NotRemovedFilter,
};
use stremio_core::models::meta_details::MetaDetails;
use stremio_core::models::player::Player;
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::types::addon::DescriptorPreview;
use stremio_core::types::resource::MetaItemPreview;
use stremio_derive::Model;

#[derive(Model, Default, Serialize)]
pub struct WebModel {
    pub ctx: Ctx<Env>,
    pub continue_watching_preview: ContinueWatchingPreview,
    pub board: CatalogsWithExtra,
    pub discover: CatalogWithFilters<MetaItemPreview>,
    pub library: LibraryWithFilters<NotRemovedFilter>,
    pub continue_watching: LibraryWithFilters<ContinueWatchingFilter>,
    pub search: CatalogsWithExtra,
    pub meta_details: MetaDetails,
    pub addons: CatalogWithFilters<DescriptorPreview>,
    pub addon_details: AddonDetails,
    pub streaming_server: StreamingServer,
    pub player: Player,
}
