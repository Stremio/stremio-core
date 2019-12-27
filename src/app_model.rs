use crate::models::LibraryItems;
use env_web::Env;
use serde::{Deserialize, Serialize};
use stremio_core::state_types::models::{
    AddonDetails, CatalogFiltered, CatalogsWithExtra, ContinueWatching, Ctx, LibraryFiltered,
    MetaDetails, Player, StreamingServerSettingsModel,
};
use stremio_core::types::addons::DescriptorPreview;
use stremio_core::types::MetaPreview;
use stremio_derive::Model;

#[derive(Model, Serialize)]
pub struct AppModel {
    pub ctx: Ctx<Env>,
    pub continue_watching: ContinueWatching,
    pub board: CatalogsWithExtra,
    pub discover: CatalogFiltered<MetaPreview>,
    pub library: LibraryFiltered,
    pub search: CatalogsWithExtra,
    pub meta_details: MetaDetails,
    pub addon_details: AddonDetails,
    pub addons: CatalogFiltered<DescriptorPreview>,
    pub streaming_server_settings: StreamingServerSettingsModel,
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
    StreamingServerSettings,
    LibraryItems,
    Player,
}
