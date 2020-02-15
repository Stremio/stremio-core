use crate::state_types::models::ctx::error::CtxError;
use crate::state_types::models::ctx::profile_loadable::Profile;
use crate::state_types::models::streaming_server::Settings as StreamingServerSettings;
use crate::state_types::EnvError;
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{Auth, AuthKey, AuthRequest};
use crate::types::{LibBucket, LibItem, UID};
use url::Url;

//
// Those messages are meant to be dispatched and hanled only inside stremio-core crate
//
#[derive(Debug)]
pub enum Internal {
    // Dispatched when some of auth, addons or settings changed.
    ProfileChanged,
    // Result for pull profile from storage.
    ProfileStorageResult(Result<Option<Profile>, CtxError>),
    // Result for pull addons from API.
    AddonsAPIResult(AuthKey, Result<Vec<Descriptor>, CtxError>),
    // Result for login/register.
    AuthResult(AuthRequest, Result<(Auth, Vec<Descriptor>), CtxError>),
    // Dispatched when some of loading status or bucket changed.
    LibraryChanged,
    // Dispatched when library item needs to be updated in the bucket, storage and API.
    UpdateLibraryItem(LibItem),
    // Result for pull library buckets from storage.
    LibraryStorageResult(Result<(Option<LibBucket>, Option<LibBucket>), CtxError>),
    // Result for pull library items from API.
    LibraryAPIResult(UID, Result<Vec<LibItem>, CtxError>),
    // Result for sync library items with API. Returns newer items that needs to be updated.
    LibrarySyncResult(UID, Result<Vec<LibItem>, CtxError>),
    // Result for loading streaming server
    StreamingServerSettingsResult(Url, Result<StreamingServerSettings, EnvError>),
    // Result for updating streaming server
    StreamingServerUpdateSettingsResult(Url, Result<(), EnvError>),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    ManifestRequestResult(TransportUrl, Result<Manifest, EnvError>),
}
