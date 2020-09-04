use crate::models::library_items::LibraryItems;
use crate::Env;
use serde::{Deserialize, Serialize};
use stremio_core::state_types::models::addon_details::AddonDetails;
use stremio_core::state_types::models::catalog_with_filters::CatalogWithFilters;
use stremio_core::state_types::models::catalogs_with_extra::CatalogsWithExtra;
use stremio_core::state_types::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::state_types::models::ctx::Ctx;
use stremio_core::state_types::models::library_with_filters::{
    ContinueWatchingFilter, LibraryWithFilters, NotRemovedFilter,
};
use stremio_core::state_types::models::meta_details::MetaDetails;
use stremio_core::state_types::models::player::Player;
use stremio_core::state_types::models::streaming_server::StreamingServer;
use stremio_core::types::addon::DescriptorPreview;
use stremio_core::types::resource::MetaItemPreview;
use stremio_derive::Model;

#[derive(Model, Default, Serialize)]
pub struct AppModel {
    pub ctx: Ctx<Env>,
    pub library_items: LibraryItems,
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

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFieldName {
    Ctx,
    LibraryItems,
    ContinueWatchingPreview,
    Board,
    Discover,
    Library,
    ContinueWatching,
    Search,
    MetaDetails,
    Addons,
    AddonDetails,
    StreamingServer,
    Player,
}
