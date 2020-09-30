use crate::env::WebEnv;
use serde::Serialize;
use stremio_core::models::addon_details::AddonDetails;
use stremio_core::models::catalog_with_filters::CatalogWithFilters;
use stremio_core::models::catalogs_with_extra::CatalogsWithExtra;
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::models::installed_addons_with_filters::InstalledAddonsWithFilters;
use stremio_core::models::library_with_filters::{
    ContinueWatchingFilter, LibraryWithFilters, NotRemovedFilter,
};
use stremio_core::models::meta_details::MetaDetails;
use stremio_core::models::player::Player;
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::runtime::Effects;
use stremio_core::types::addon::DescriptorPreview;
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::profile::Profile;
use stremio_core::types::resource::MetaItemPreview;
use stremio_derive::Model;

#[derive(Model, Serialize)]
pub struct WebModel {
    pub ctx: Ctx<WebEnv>,
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
    pub fn new(profile: Profile, library: LibraryBucket) -> (WebModel, Effects) {
        let (discover, discover_effects) = CatalogWithFilters::<MetaItemPreview>::new(&profile);
        let (remote_addons, remote_addons_effects) =
            CatalogWithFilters::<DescriptorPreview>::new(&profile);
        let (installed_addons, installed_addons_effects) =
            InstalledAddonsWithFilters::new(&profile);
        let (streaming_server, streaming_server_effects) = StreamingServer::new::<WebEnv>(&profile);
        let model = WebModel {
            ctx: Ctx::new(profile, library),
            discover,
            remote_addons,
            installed_addons,
            streaming_server,
            continue_watching_preview: Default::default(),
            board: Default::default(),
            library: Default::default(),
            continue_watching: Default::default(),
            search: Default::default(),
            meta_details: Default::default(),
            addon_details: Default::default(),
            player: Default::default(),
        };
        (
            model,
            discover_effects
                .join(remote_addons_effects)
                .join(installed_addons_effects)
                .join(streaming_server_effects),
        )
    }
}
