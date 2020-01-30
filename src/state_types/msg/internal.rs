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
    // Dispatched when some of the user data changed.
    UserChanged,
    // Dispatched when user is pulled from storage.
    UserStorageResult(Result<Option<User>, CtxError>),
    // Dispatched when user is authenticated.
    UserAuthenticateResult(APIRequest, Result<(Auth, Vec<Descriptor>), CtxError>),
    // Dispatched when addons are pulled from API.
    UserPullAddonsResult(AuthKey, Result<Vec<Descriptor>, CtxError>),
    LibraryChanged,
    LibraryStorageResult(Result<(Option<LibBucket>, Option<LibBucket>), CtxError>),
    LibraryAPIResult(UID, Result<Vec<LibItem>, CtxError>),
    LibrarySyncResult(UID, Result<Vec<LibItem>, CtxError>),
    UpdateLibraryItem(LibItem),
    StreamingServerReloadResult(Url, Result<(Url, StreamingServerSettings), EnvError>),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    ManifestRequestResult(TransportUrl, Result<Manifest, EnvError>),
}
