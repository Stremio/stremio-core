use crate::state_types::models::ctx::error::CtxError;
use crate::state_types::models::ctx::user::User;
use crate::state_types::models::streaming_server::Settings as StreamingServerSettings;
use crate::state_types::EnvError;
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{APIRequest, Auth, AuthKey};
use crate::types::{LibBucket, LibItem, UID};
use url::Url;

//
// Those messages are meant to be dispatched and hanled only inside stremio-core crate
//
#[derive(Debug)]
pub enum Internal {
    // Dispatched when some of auth, addons or settings changed.
    UserChanged,
    // Result for pull user from storage.
    UserStorageResult(Result<Option<User>, CtxError>),
    // Result for login/register.
    UserAuthResult(APIRequest, Result<(Auth, Vec<Descriptor>), CtxError>),
    // Result for pull user addons from API.
    UserAddonsAPIResult(AuthKey, Result<Vec<Descriptor>, CtxError>),
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
    StreamingServerReloadResult(Url, Result<(Url, StreamingServerSettings), EnvError>),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    ManifestRequestResult(TransportUrl, Result<Manifest, EnvError>),
}
