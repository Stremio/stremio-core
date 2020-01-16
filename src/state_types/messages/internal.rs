use super::MsgError;
use crate::state_types::models::{SsSettings, UserData};
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{APIRequest, AuthKey, DatastoreReq};
use crate::types::{LibBucket, UID};

#[derive(Debug)]
pub enum Internal {
    UserDataChanged,
    UserDataStorageResult(Box<Option<UserData>>),
    UserDataRequestResponse(APIRequest, Box<UserData>),
    LibraryChanged,
    LibraryStorageResponse(UID, Box<Option<LibBucket>>, Box<Option<LibBucket>>),
    LibraryAPIResponse(Box<LibBucket>),
    LibrarySyncResponse(Box<LibBucket>),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, MsgError>>),
    ManifestRequestResult(TransportUrl, Box<Result<Manifest, MsgError>>),
    AddonsRequestResponse(AuthKey, Box<Vec<Descriptor>>),
    StreamingServerSettingsLoaded(SsSettings),
    StreamingServerSettingsErrored(String),
}
