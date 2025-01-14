use url::Url;

use crate::models::common::ResourceLoadable;
use crate::models::ctx::CtxError;
use crate::models::link::LinkError;
use crate::models::local_search::Searchable;
use crate::models::streaming_server::PlaybackDevice;
use crate::runtime::EnvError;
use crate::types::addon::{Descriptor, Manifest, ResourceRequest, ResourceResponse};
use crate::types::api::{
    APIRequest, AuthRequest, DataExportResponse, DatastoreRequest, GetModalResponse,
    GetNotificationResponse, LinkCodeResponse, LinkDataResponse, SeekLogRequest, SkipGapsRequest,
    SkipGapsResponse, SuccessResponse,
};
use crate::types::library::{LibraryBucket, LibraryItem, LibraryItemId};
use crate::types::profile::{Auth, AuthKey, Profile, User};
use crate::types::streaming_server::{
    DeviceInfo, GetHTTPSResponse, NetworkInfo, SettingsResponse, Statistics, StatisticsRequest,
};
use crate::types::streams::StreamItemState;
use crate::types::{
    resource::{MetaItem, Stream},
    torrent::InfoHash,
};

pub type CtxStorageResponse = (
    Option<Profile>,
    Option<LibraryBucket>,
    Option<LibraryBucket>,
);

#[derive(Debug)]
pub struct CtxAuthResponse {
    pub auth: Auth,
    pub addons_result: Result<Vec<Descriptor>, CtxError>,
    pub library_items_result: Result<Vec<LibraryItem>, CtxError>,
}

pub type LibraryPlanResponse = (Vec<String>, Vec<String>);

//
// Those messages are meant to be dispatched and handled only inside stremio-core crate
//
#[derive(Debug)]
pub enum Internal {
    /// Result for authenticate to API.
    CtxAuthResult(AuthRequest, Result<CtxAuthResponse, CtxError>),
    /// Result for pull addons from API.
    AddonsAPIResult(APIRequest, Result<Vec<Descriptor>, CtxError>),
    /// Result for pull user from API.
    UserAPIResult(APIRequest, Result<User, CtxError>),
    /// Result for deleting account from API.
    DeleteAccountAPIResult(APIRequest, Result<SuccessResponse, CtxError>),
    /// Result for library sync plan with API.
    LibrarySyncPlanResult(DatastoreRequest, Result<LibraryPlanResponse, CtxError>),
    /// Result for pull library items from API.
    LibraryPullResult(DatastoreRequest, Result<Vec<LibraryItem>, CtxError>),
    /// Dispatched when the user session needs to be cleared with a flag if the session was already deleted server-side
    Logout(bool),
    /// Internal event dispatched on user action or login
    /// to install the addon if it's not present
    InstallTraktAddon,
    /// Dispatched when addons needs to be installed.
    InstallAddon(Descriptor),
    /// Dispatched when addons needs to be uninstalled.
    UninstallAddon(Descriptor),
    UninstallTraktAddon,
    /// Dispatched when a new stream is loaded into the Player.
    StreamLoaded {
        stream: Stream,
        stream_request: Option<ResourceRequest>,
        meta_item: ResourceLoadable<MetaItem>,
    },
    /// Dispatched when stream item's state has changed
    StreamStateChanged {
        state: StreamItemState,
        stream_request: Option<ResourceRequest>,
        meta_request: Option<ResourceRequest>,
    },
    /// Dispatched when requesting search on catalogs.
    CatalogsWithExtraSearch {
        query: String,
    },
    /// Dispatched when library item needs to be updated in the memory, storage and API.
    UpdateLibraryItem(LibraryItem),
    /// Dispatched when some of auth, addons or settings changed.
    ProfileChanged,
    /// Dispatched when library changes with a flag if its already persisted.
    LibraryChanged(bool),
    /// Dispatched when streams bucket changes with a flag if its already persisted.
    StreamsChanged(bool),
    /// Search history has changed.
    SearchHistoryChanged,
    /// Server URLs bucket has changed.
    StreamingServerUrlsBucketChanged,
    /// User notifications have changed
    NotificationsChanged,
    /// Pulling of notifications triggered either by the user (with an action) or
    /// internally in core.
    PullNotifications,
    /// Dismiss all Notifications for a given [`MetaItemId`].
    ///
    /// [`MetaItemId`]: crate::types::resource::MetaItemId
    DismissNotificationItem(LibraryItemId),
    /// Result for loading link code.
    LinkCodeResult(Result<LinkCodeResponse, LinkError>),
    /// Result for loading link data.
    LinkDataResult(String, Result<LinkDataResponse, LinkError>),
    /// Result for loading streaming server settings.
    StreamingServerSettingsResult(Url, Result<SettingsResponse, EnvError>),
    /// Result for loading streaming server base url.
    StreamingServerBaseURLResult(Url, Result<Url, EnvError>),
    // Result for loading streaming server playback devices.
    StreamingServerPlaybackDevicesResult(Url, Result<Vec<PlaybackDevice>, EnvError>),
    // Result for network info.
    StreamingServerNetworkInfoResult(Url, Result<NetworkInfo, EnvError>),
    // Result for device info.
    StreamingServerDeviceInfoResult(Url, Result<DeviceInfo, EnvError>),
    /// Result for updating streaming server settings.
    StreamingServerUpdateSettingsResult(Url, Result<(), EnvError>),
    /// Result for creating a torrent.
    StreamingServerCreateTorrentResult(InfoHash, Result<(), EnvError>),
    /// Result for playing on device.
    StreamingServerPlayOnDeviceResult(String, Result<(), EnvError>),
    // Result for get https endpoint request
    StreamingServerGetHTTPSResult(Url, Result<GetHTTPSResponse, EnvError>),
    /// Result for streaming server statistics.
    ///
    /// Server will return None (or `null`) in response for [`Statistics`]`,
    /// when stream has been fully loaded up to 100%
    StreamingServerStatisticsResult(
        (Url, StatisticsRequest),
        Result<Option<Statistics>, EnvError>,
    ),
    /// Result for fetching resource from addons.
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    /// Result for fetching manifest from addon.
    ManifestRequestResult(Url, Result<Manifest, EnvError>),
    /// TODO: write some obvious comment about what it is
    NotificationsRequestResult(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    /// Result for requesting a `dataExport` of user data.
    DataExportResult(AuthKey, Result<DataExportResponse, CtxError>),
    /// Result for submitting SeekLogs request for a played stream.
    ///
    /// Applicable only to movie series and torrents.
    SeekLogsResult(SeekLogRequest, Result<SuccessResponse, CtxError>),
    /// Retrieve the skip gaps for skipping intro and outro.
    SkipGapsResult(SkipGapsRequest, Result<SkipGapsResponse, CtxError>),
    /// The result of querying the data for LocalSearch
    LoadLocalSearchResult(Url, Result<Vec<Searchable>, EnvError>),
    /// Result for getModal request
    GetModalResult(APIRequest, Result<Option<GetModalResponse>, CtxError>),
    /// Result for getNotification request
    GetNotificationResult(
        APIRequest,
        Result<Option<GetNotificationResponse>, CtxError>,
    ),
    /// When dismissed events changed
    DismissedEventsChanged,
}
