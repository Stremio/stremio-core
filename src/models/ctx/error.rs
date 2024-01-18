use crate::runtime::EnvError;
use crate::types::api::APIError;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

// TODO move this to runtime::msg::Error and rename it
#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(tag = "type")]
pub enum CtxError {
    API(APIError),
    Env(EnvError),
    Other(OtherError),
}

impl From<APIError> for CtxError {
    fn from(error: APIError) -> Self {
        CtxError::API(error)
    }
}

impl From<EnvError> for CtxError {
    fn from(error: EnvError) -> Self {
        CtxError::Env(error)
    }
}

impl From<OtherError> for CtxError {
    fn from(error: OtherError) -> Self {
        CtxError::Other(error)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OtherError {
    UserNotLoggedIn,
    LibraryItemNotFound,
    AddonAlreadyInstalled,
    AddonNotInstalled,
    AddonIsProtected,
    AddonConfigurationRequired,
    UserAddonsAreLocked,
    UserLibraryIsMissing,
}

impl OtherError {
    pub fn message(&self) -> String {
        match &self {
            OtherError::UserNotLoggedIn => "User is not logged in".to_owned(),
            OtherError::LibraryItemNotFound => "Item is not found in library".to_owned(),
            OtherError::AddonAlreadyInstalled => "Addon is already installed".to_owned(),
            OtherError::AddonNotInstalled => "Addon is not installed".to_owned(),
            OtherError::AddonIsProtected => "Addon is protected".to_owned(),
            OtherError::AddonConfigurationRequired => "Addon requires configuration".to_owned(),
            OtherError::UserAddonsAreLocked => "Fetching Addons from the API failed and we have defaulted the addons to the officials ones until the request succeeds".to_owned(),
            OtherError::UserLibraryIsMissing => "Fetching Library from the API failed and we have defaulted to empty library until the request succeeds".to_owned(),
        }
    }
    pub fn code(&self) -> u64 {
        match &self {
            OtherError::UserNotLoggedIn => 1,
            OtherError::LibraryItemNotFound => 2,
            OtherError::AddonAlreadyInstalled => 3,
            OtherError::AddonNotInstalled => 4,
            OtherError::AddonIsProtected => 5,
            OtherError::AddonConfigurationRequired => 6,
            OtherError::UserAddonsAreLocked => 7,
            OtherError::UserLibraryIsMissing => 8,
        }
    }
}

impl Serialize for OtherError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("OtherError", 2)?;
        state.serialize_field("code", &self.code())?;
        state.serialize_field("message", &self.message())?;
        state.end()
    }
}
