use crate::constants::{OFFICIAL_ADDONS, USER_DATA_KEY};
use crate::state_types::messages::{
    Action, ActionAddon, ActionLoad, ActionSettings, ActionUser, Event, Internal, Msg, MsgError,
};
use crate::state_types::models::common::api_fetch;
use crate::state_types::{Effect, Effects, Environment};
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, Auth, AuthResponse, CollectionResponse};
use derivative::Derivative;
use futures::future::Either;
use futures::{future, Future};
use serde::de::DeserializeOwned;
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
            streaming_server_url: Url::parse("http://127.0.0.1:11470").unwrap(),
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
pub struct UserData {
    pub auth: Option<Auth>,
    pub addons: Vec<Descriptor>,
    pub settings: Settings,
}

impl Default for UserData {
    fn default() -> Self {
        UserData {
            auth: None,
            addons: OFFICIAL_ADDONS.to_owned(),
            settings: Settings::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum UserDataRequest {
    APIRequest(APIRequest),
    StorageRequest,
}

#[derive(Derivative, Clone, Debug, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(tag = "type")]
pub enum UserDataLoadable {
    #[derivative(Default)]
    NotLoaded { content: UserData },
    Loading {
        request: UserDataRequest,
        content: UserData,
    },
    Ready {
        request: UserDataRequest,
        content: UserData,
    },
}

impl UserDataLoadable {
    pub fn update<Env: Environment + 'static>(&mut self, msg: &Msg) -> Effects {
        let content = match &mut self {
            UserDataLoadable::NotLoaded { content }
            | UserDataLoadable::Loading { content, .. }
            | UserDataLoadable::Ready { content, .. } => content,
        };
        match msg {
            Msg::Action(Action::Load(ActionLoad::UserData)) => match &self {
                UserDataLoadable::NotLoaded { .. } => {
                    *self = UserDataLoadable::Loading {
                        request: UserDataRequest::StorageRequest,
                        content: UserData::default(),
                    };
                    Effects::one(load_user_data::<Env>())
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Action(Action::AddonOp(action_addon)) => match action_addon {
                ActionAddon::Install(descriptor) => {
                    let position = content
                        .addons
                        .iter()
                        .position(|addon| addon.transport_url.eq(&descriptor.transport_url));
                    if let Some(position) = position {
                        content.addons.remove(position);
                    };
                    content.addons.push(descriptor.to_owned());
                    Effects::one(save_user_data::<Env>(&content))
                }
                ActionAddon::Uninstall { transport_url } => {
                    let position = content.addons.iter().position(|addon| {
                        addon.transport_url.eq(transport_url) && !addon.flags.protected
                    });
                    match position {
                        Some(position) => {
                            content.addons.remove(position);
                            Effects::one(save_user_data::<Env>(&content))
                        }
                        _ => Effects::none().unchanged(),
                    }
                }
            },
            Msg::Action(Action::Settings(action_settings)) => match action_settings {
                ActionSettings::UpdateSettings(settings) => {
                    content.settings = settings.to_owned();
                    Effects::one(save_user_data::<Env>(&content))
                }
            },
            Msg::Action(Action::UserOp(action_user)) => match action_user {
                ActionUser::Login { email, password } => {
                    let api_request = APIRequest::Login {
                        email: email.to_owned(),
                        password: password.to_owned(),
                    };
                    let next_content = UserData::default();
                    *self = UserDataLoadable::Loading {
                        request: UserDataRequest::APIRequest(api_request.to_owned()),
                        content: next_content.to_owned(),
                    };
                    Effects::one(save_user_data::<Env>(&next_content)).join(Effects::one(
                        request_user_data::<Env, _, _>(
                            &api_request,
                            action_user,
                            |AuthResponse { key, user }| Some(Auth { key, user }),
                        ),
                    ))
                }
                ActionUser::Register {
                    email,
                    password,
                    gdpr_consent,
                } => {
                    let api_request = APIRequest::Register {
                        email: email.to_owned(),
                        password: password.to_owned(),
                        gdpr_consent: gdpr_consent.to_owned(),
                    };
                    let next_content = UserData::default();
                    *self = UserDataLoadable::Loading {
                        request: UserDataRequest::APIRequest(api_request.to_owned()),
                        content: next_content.to_owned(),
                    };
                    Effects::one(save_user_data::<Env>(&next_content)).join(Effects::one(
                        request_user_data::<Env, _, _>(
                            &api_request,
                            action_user,
                            |AuthResponse { key, user }| Some(Auth { key, user }),
                        ),
                    ))
                }
                ActionUser::Logout => match &content.auth {
                    Some(auth) => {
                        let api_request = APIRequest::Logout {
                            auth_key: auth.key.to_owned(),
                        };
                        let next_content = UserData::default();
                        *self = UserDataLoadable::Loading {
                            request: UserDataRequest::APIRequest(api_request.to_owned()),
                            content: next_content.to_owned(),
                        };
                        Effects::one(save_user_data::<Env>(&next_content)).join(Effects::one(
                            request_user_data::<Env, _, _>(&api_request, action_user, |_| None),
                        ))
                    }
                    _ => Effects::none().unchanged(),
                },
            },
            Msg::Internal(Internal::UserDataLoaded(user_data)) => match &self {
                UserDataLoadable::Loading {
                    request: UserDataRequest::StorageRequest,
                    ..
                } => {
                    *self = UserDataLoadable::Ready {
                        request: UserDataRequest::StorageRequest,
                        content: user_data.to_owned().unwrap_or_default(),
                    };
                    Effects::msg(Msg::Event(Event::UserDataChanged))
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserDataResponse(api_request, user_data)) => match &self {
                UserDataLoadable::Loading { request, .. }
                    if request.eq(&UserDataRequest::APIRequest(api_request.to_owned())) =>
                {
                    let next_content = user_data.to_owned();
                    *self = UserDataLoadable::Ready {
                        request: UserDataRequest::APIRequest(api_request.to_owned()),
                        content: next_content,
                    };
                    Effects::msg(Msg::Event(Event::UserDataChanged))
                        .join(Effects::one(save_user_data::<Env>(&next_content)))
                }
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        }
    }
}

fn load_user_data<Env: Environment + 'static>() -> Effect {
    Box::new(
        Env::get_storage(USER_DATA_KEY)
            .map(|user_data| Msg::Internal(Internal::UserDataLoaded(user_data)))
            .map_err(|error| Msg::Event(Event::StorageError(MsgError::from(error)))),
    )
}

fn save_user_data<Env: Environment + 'static>(user_data: &UserData) -> Effect {
    Box::new(
        Env::set_storage(USER_DATA_KEY, Some(user_data))
            .map(|_| Msg::Event(Event::UserDataSaved))
            .map_err(|error| Msg::Event(Event::StorageError(MsgError::from(error)))),
    )
}

fn request_user_data<Env: Environment + 'static, Response, MapResponseToAuth>(
    api_request: &APIRequest,
    action_user: &ActionUser,
    map_response_to_auth: MapResponseToAuth,
) -> Effect
where
    MapResponseToAuth: FnOnce(Response) -> Option<Auth> + 'static,
    Response: DeserializeOwned + 'static,
{
    let api_request = api_request.to_owned();
    let action_user = action_user.to_owned();
    Box::new(
        api_fetch::<Env, Response, _>(api_request.to_owned())
            .map(map_response_to_auth)
            .and_then(|auth| match auth {
                Some(auth) => Either::A(pull_addons::<Env>(&auth).map(
                    move |CollectionResponse { addons, .. }| UserData {
                        auth: Some(auth),
                        addons,
                        settings: Settings::default(),
                    },
                )),
                _ => Either::B(future::ok(UserData::default())),
            })
            .map(move |user_data| Msg::Internal(Internal::UserDataResponse(api_request, user_data)))
            .map_err(move |error| Msg::Event(Event::UserActionError(action_user, error))),
    )
}

fn pull_addons<Env: Environment + 'static>(
    auth: &Auth,
) -> impl Future<Item = CollectionResponse, Error = MsgError> {
    let auth = auth.to_owned();
    let pull_addons_request = APIRequest::AddonCollectionGet {
        auth_key: auth.key.to_owned(),
        update: true,
    };
    api_fetch::<Env, _, _>(pull_addons_request)
}
