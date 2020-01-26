use super::MsgError;
use crate::state_types::models::ctx::UserData;
// use crate::state_types::models::settings::SsSettings;
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{APIRequest, Auth, AuthKey};
use crate::types::{LibBucket, LibItem, UID};

//
// Those messages are meant to be dispatched and hanled only inside stremio-core crate
//
#[derive(Debug)]
pub enum Internal {
    UserDataChanged,
    UserDataStorageResponse(Option<UserData>),
    UserAuthResponse(APIRequest, Auth),
    UserAddonsResponse(AuthKey, Vec<Descriptor>),
    LibraryChanged,
    LibraryStorageResult(
        UID,
        Result<(Option<LibBucket>, Option<LibBucket>), MsgError>,
    ),
    LibraryAPIResult(UID, Result<Vec<LibItem>, MsgError>),
    LibrarySyncResponse(LibBucket),
    UpdateLibraryItem(LibItem),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, MsgError>>),
    ManifestRequestResult(TransportUrl, Result<Manifest, MsgError>),
    // StreamingServerSettingsLoaded(SsSettings),
    // StreamingServerSettingsErrored(String),
}
