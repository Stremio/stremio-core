use crate::state_types::models::common::ModelError;
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
    UserDataStorageResult(Result<Option<UserData>, ModelError>),
    UserAuthResult(APIRequest, Result<Auth, ModelError>),
    UserAddonsResult(AuthKey, Result<Vec<Descriptor>, ModelError>),
    LibraryChanged,
    LibraryStorageResult(
        // todo remove uid
        UID,
        Result<(Option<LibBucket>, Option<LibBucket>), ModelError>,
    ),
    LibraryAPIResult(UID, Result<Vec<LibItem>, ModelError>),
    LibrarySyncResponse(LibBucket),
    UpdateLibraryItem(LibItem),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, ModelError>>),
    ManifestRequestResult(TransportUrl, Result<Manifest, ModelError>),
    // StreamingServerSettingsLoaded(SsSettings),
    // StreamingServerSettingsErrored(String),
}
