use crate::state_types::models::{CtxContent, SsSettings};
use crate::state_types::EnvError;
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::AuthKey;
use crate::types::LibBucket;

#[derive(Debug)]
pub enum Internal {
    // Context loaded from storage
    CtxLoaded(Option<Box<CtxContent>>),
    // Context updated as a result of authentication/logout
    CtxUpdate(Box<CtxContent>),
    // Context addons pulled
    // this should replace ctx.content.addons entirely
    CtxAddonsPulled(AuthKey, Vec<Descriptor>),
    // Library is loaded, either from storage or from an initial API sync
    // This will replace the whole library index, as long as bucket UID matches
    LibLoaded(LibBucket),
    // Library: pulled some newer items from the API
    // this will extend the current index of libitems, it doesn't replace it
    LibSyncPulled(LibBucket),
    // Response from an add-on
    AddonResponse(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    ManifestResponse(TransportUrl, Box<Result<Manifest, EnvError>>),
    StreamingServerSettingsLoaded(SsSettings),
    StreamingServerSettingsErrored(String),
}
