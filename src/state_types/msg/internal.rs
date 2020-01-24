use super::MsgError;
use crate::state_types::models::ctx::UserData;
// use crate::state_types::models::settings::SsSettings;
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{APIRequest, Auth, AuthKey};
use crate::types::{LibBucket, LibItem, UID};

#[derive(Debug)]
pub enum Internal {
    UserDataChanged,
    UserDataStorageResponse(Box<Option<UserData>>),
    UserAuthResponse(APIRequest, Box<Auth>),
    UserAddonsResponse(AuthKey, Vec<Descriptor>),
    LibraryChanged,
    LibraryStorageResponse(UID, Box<Option<LibBucket>>, Box<Option<LibBucket>>),
    LibraryAPIResponse(UID, Vec<LibItem>),
    LibrarySyncResponse(Box<LibBucket>),
    UpdateLibraryItem(Box<LibItem>),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, MsgError>>),
    ManifestRequestResult(TransportUrl, Box<Result<Manifest, MsgError>>),
    // StreamingServerSettingsLoaded(SsSettings),
    // StreamingServerSettingsErrored(String),
}
