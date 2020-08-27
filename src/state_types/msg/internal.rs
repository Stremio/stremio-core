use crate::state_types::models::ctx::CtxError;
use crate::state_types::models::streaming_server::Settings as StreamingServerSettings;
use crate::state_types::EnvError;
use crate::types::addon::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{APIRequest, AuthRequest, DatastoreRequest};
use crate::types::profile::{Profile, UID, Auth};
use crate::types::library::LibItem;
use url::Url;

pub type CtxStorageResponse = (
    Option<Profile>,
    Option<(UID, Vec<LibItem>)>,
    Option<(UID, Vec<LibItem>)>,
);

pub type AuthResponse = (Auth, Vec<Descriptor>, Vec<LibItem>);

pub type LibraryPlanResponse = (Vec<String>, Vec<String>);

//
// Those messages are meant to be dispatched and hanled only inside stremio-core crate
//
#[derive(Debug)]
pub enum Internal {
    // Result for pull profile and library from storage.
    CtxStorageResult(Result<CtxStorageResponse, CtxError>),
    // Result for authenticate to API.
    CtxAuthResult(AuthRequest, Result<AuthResponse, CtxError>),
    // Result for pull addons from API.
    AddonsAPIResult(APIRequest, Result<Vec<Descriptor>, CtxError>),
    // Result for library sync plan with API.
    LibrarySyncPlanResult(DatastoreRequest, Result<LibraryPlanResponse, CtxError>),
    // Result for pull library items from API.
    LibraryPullResult(DatastoreRequest, Result<Vec<LibItem>, CtxError>),
    // Dispatched when library item needs to be updated in the memory, storage and API.
    UpdateLibraryItem(LibItem),
    // Dispatched when some of auth, addons or settings changed with a flag if its already persisted.
    ProfileChanged(bool),
    // Dispatched when library changes with a flag if its already persisted.
    LibraryChanged(bool),
    // Result for loading streaming server settings.
    StreamingServerSettingsResult(Url, Result<StreamingServerSettings, EnvError>),
    // Result for loading streaming server base url.
    StreamingServerBaseURLResult(Url, Result<Url, EnvError>),
    // Result for updating streaming server settings.
    StreamingServerUpdateSettingsResult(Url, Result<(), EnvError>),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    ManifestRequestResult(TransportUrl, Result<Manifest, EnvError>),
}
