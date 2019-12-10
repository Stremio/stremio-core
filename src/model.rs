use env_web::Env;
use serde::{Deserialize, Serialize};
use stremio_core::state_types::models::{
    CatalogFiltered, CatalogsWithExtra, ContinueWatching, Ctx, LibraryFiltered, MetaDetails,
    StreamingServerSettingsModel,
};
use stremio_core::types::addons::DescriptorPreview;
use stremio_core::types::MetaPreview;
use stremio_derive::Model;

#[derive(Model, Serialize)]
pub struct Model {
    pub ctx: Ctx<Env>,
    pub continue_watching: ContinueWatching,
    pub board: CatalogsWithExtra,
    pub discover: CatalogFiltered<MetaPreview>,
    pub library: LibraryFiltered,
    pub search: CatalogsWithExtra,
    pub meta_details: MetaDetails,
    pub addons: CatalogFiltered<DescriptorPreview>,
    pub streaming_server_settings: StreamingServerSettingsModel,
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
    Addons,
    StreamingServerSettings,
}
