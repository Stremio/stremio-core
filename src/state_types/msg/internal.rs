use super::MsgError;
use crate::state_types::models::ctx::UserData;
// use crate::state_types::models::settings::SsSettings;
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{APIRequest, Auth, AuthKey};
use crate::types::{LibBucket, LibItem, UID};

#[derive(Debug)]
pub enum Internal {
    UserDataChanged,
    UserDataStorageResponse(Option<UserData>),
    UserAuthResponse(APIRequest, Auth),
    UserAddonsResponse(AuthKey, Vec<Descriptor>),
    LibraryChanged,
    LibraryStorageResponse(UID, Option<LibBucket>, Option<LibBucket>),
    LibraryAPIResponse(UID, Vec<LibItem>),
    LibrarySyncResponse(LibBucket),
    UpdateLibraryItem(LibItem),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, MsgError>>),
    ManifestRequestResult(TransportUrl, Result<Manifest, MsgError>),
    // StreamingServerSettingsLoaded(SsSettings),
    // StreamingServerSettingsErrored(String),
}
