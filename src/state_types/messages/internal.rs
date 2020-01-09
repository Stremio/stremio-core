use super::MsgError;
use crate::state_types::models::{SsSettings, UserData};
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{APIRequest, AuthKey};
use crate::types::LibBucket;

#[derive(Debug)]
pub enum Internal {
    UserDataLoaded(Option<UserData>),
    UserDataResponse(APIRequest, UserData),
    AddonResponse(ResourceRequest, Result<ResourceResponse, MsgError>),
    ManifestResponse(TransportUrl, Result<Manifest, MsgError>),
    // Context addons pulled
    // this should replace ctx.content.addons entirely
    CtxAddonsPulled(AuthKey, Vec<Descriptor>),
    // Library is loaded, either from storage or from an initial API sync
    // This will replace the whole library index, as long as bucket UID matches
    LibLoaded(LibBucket),
    // Library: pulled some newer items from the API
    // this will extend the current index of libitems, it doesn't replace it
    LibSyncPulled(LibBucket),
    StreamingServerSettingsLoaded(SsSettings),
    StreamingServerSettingsErrored(String),
}
