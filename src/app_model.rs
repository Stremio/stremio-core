use crate::models::library_items::LibraryItems;
use env_web::Env;
use serde::{Deserialize, Serialize};
use stremio_core::state_types::models::addon_details::AddonDetails;
use stremio_core::state_types::models::catalog_with_filters::CatalogWithFilters;
use stremio_core::state_types::models::catalogs_with_extra::CatalogsWithExtra;
use stremio_core::state_types::models::continue_watching::ContinueWatching;
use stremio_core::state_types::models::ctx::Ctx;
use stremio_core::state_types::models::library_filtered::LibraryFiltered;
use stremio_core::state_types::models::meta_details::MetaDetails;
use stremio_core::state_types::models::player::Player;
use stremio_core::state_types::models::streaming_server::StreamingServerLoadable;
use stremio_core::types::addons::DescriptorPreview;
use stremio_core::types::MetaPreview;
use stremio_derive::Model;

#[derive(Model, Default, Serialize)]
pub struct AppModel {
    pub ctx: Ctx<Env>,
    pub continue_watching: ContinueWatching,
    pub board: CatalogsWithExtra,
    pub discover: CatalogWithFilters<MetaPreview>,
    pub library: LibraryFiltered,
    pub search: CatalogsWithExtra,
    pub meta_details: MetaDetails,
    pub addon_details: AddonDetails,
    pub addons: CatalogWithFilters<DescriptorPreview>,
    pub streaming_server: StreamingServerLoadable,
    pub library_items: LibraryItems,
    pub player: Player,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFieldName {
    Ctx,
    ContinueWatching,
    Board,
    Discover,
    Library,
    Search,
    MetaDetails,
    AddonDetails,
    Addons,
    StreamingServer,
    LibraryItems,
    Player,
}
