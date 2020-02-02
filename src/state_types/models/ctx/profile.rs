use super::error::CtxError;
use super::fetch_api;
use crate::constants::{OFFICIAL_ADDONS, PROFILE_STORAGE_KEY, STREAMING_SERVER_URL};
use crate::state_types::Environment;
use crate::types::addons::Descriptor;
use crate::types::api::{
    APIRequest, Auth, AuthKey, AuthResponse, CollectionResponse, SuccessResponse,
};
use derivative::Derivative;
use futures::Future;
use serde::{Deserialize, Serialize};
use url::Url;
use url_serde;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub interface_language: String,
    #[serde(with = "url_serde")]
    pub streaming_server_url: Url,
    pub binge_watching: bool,
    pub play_in_background: bool,
    pub play_in_external_player: bool,
    pub subtitles_language: String,
    pub subtitles_size: u8,
    pub subtitles_text_color: String,
    pub subtitles_background_color: String,
    pub subtitles_outline_color: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            binge_watching: false,
            play_in_background: true,
            play_in_external_player: false,
            streaming_server_url: Url::parse(STREAMING_SERVER_URL)
                .expect("streaming_server_url builder cannot fail"),
            interface_language: "eng".to_owned(),
            subtitles_language: "eng".to_owned(),
            subtitles_size: 2,
            subtitles_text_color: "#FFFFFF00".to_owned(),
            subtitles_background_color: "#00000000".to_owned(),
            subtitles_outline_color: "#00000000".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub auth: Option<Auth>,
    pub addons: Vec<Descriptor>,
    pub settings: Settings,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            auth: None,
            addons: OFFICIAL_ADDONS.to_owned(),
            settings: Settings::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProfileRequest {
    Storage,
    API(APIRequest),
}

#[derive(Derivative, Clone, Debug, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(untagged)]
pub enum ProfileLoadable {
    Loading {
        #[serde(skip)]
        request: ProfileRequest,
        #[serde(flatten)]
        content: Profile,
    },
    #[derivative(Default)]
    Ready {
        #[serde(flatten)]
        content: Profile,
    },
}

impl ProfileLoadable {
    pub fn content(&self) -> &Profile {
        match &self {
            ProfileLoadable::Loading { content, .. } | ProfileLoadable::Ready { content } => {
                content
            }
        }
    }
    pub fn content_mut(&mut self) -> &mut Profile {
        match self {
            ProfileLoadable::Loading { content, .. } | ProfileLoadable::Ready { content } => {
                content
            }
        }
    }
    pub fn pull_from_storage<Env: Environment + 'static>(
    ) -> impl Future<Item = Option<Profile>, Error = CtxError> {
        Env::get_storage(PROFILE_STORAGE_KEY).map_err(CtxError::from)
    }
    pub fn push_to_storage<Env: Environment + 'static>(
        profile: Option<&Profile>,
    ) -> impl Future<Item = (), Error = CtxError> {
        Env::set_storage(PROFILE_STORAGE_KEY, profile).map_err(CtxError::from)
    }
    pub fn authenticate<Env: Environment + 'static>(
        request: &APIRequest,
    ) -> impl Future<Item = (Auth, Vec<Descriptor>), Error = CtxError> {
        fetch_api::<Env, _, _>(request)
            .map(|AuthResponse { key, user }| Auth { key, user })
            .and_then(|auth| {
                fetch_api::<Env, _, _>(&APIRequest::AddonCollectionGet {
                    auth_key: auth.key.to_owned(),
                    update: true,
                })
                .map(move |CollectionResponse { addons, .. }| (auth, addons))
            })
    }
    pub fn delete_session<Env: Environment + 'static>(
        auth_key: &AuthKey,
    ) -> impl Future<Item = (), Error = CtxError> {
        fetch_api::<Env, _, SuccessResponse>(&APIRequest::Logout {
            auth_key: auth_key.to_owned(),
        })
        .map(|_| ())
    }
    pub fn pull_user_from_api<Env: Environment + 'static>(auth_key: &AuthKey) {
        unimplemented!();
    }
    pub fn push_user_to_api<Env: Environment + 'static>(auth_key: &AuthKey) {
        unimplemented!();
    }
    pub fn pull_addons_from_api<Env: Environment + 'static>(
        auth_key: &AuthKey,
        update: bool,
    ) -> impl Future<Item = Vec<Descriptor>, Error = CtxError> {
        fetch_api::<Env, _, _>(&APIRequest::AddonCollectionGet {
            auth_key: auth_key.to_owned(),
            update,
        })
        .map(|CollectionResponse { addons, .. }| addons)
    }
    pub fn push_addons_to_api<Env: Environment + 'static>(
        auth_key: &AuthKey,
        addons: &[Descriptor],
    ) -> impl Future<Item = (), Error = CtxError> {
        fetch_api::<Env, _, SuccessResponse>(&APIRequest::AddonCollectionSet {
            auth_key: auth_key.to_owned(),
            addons: addons.to_owned(),
        })
        .map(|_| ())
    }
}
