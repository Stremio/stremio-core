use crate::constants::{OFFICIAL_ADDONS, USER_DATA_KEY};
use crate::state_types::messages::{
    Action, ActionAddons, ActionCtx, ActionSettings, ActionUser, Event, Internal, Msg, MsgError,
};
use crate::state_types::models::common::{authenticate, fetch_api, get_addons};
use crate::state_types::{Effects, Environment};
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, Auth};
use derivative::Derivative;
use futures::Future;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
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
            streaming_server_url: Url::parse("http://127.0.0.1:11470")
                .expect("builder cannot fail"),
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
    Storage,
    API(APIRequest),
}

#[derive(Derivative, Clone, Debug, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(tag = "type")]
pub enum UserDataLoadable {
    Loading {
        request: UserDataRequest,
        content: UserData,
    },
    #[derivative(Default)]
    Ready { content: UserData },
}

impl UserDataLoadable {
    pub fn update<Env: Environment + 'static>(&mut self, msg: &Msg) -> Effects {
        let user_data = match &mut self {
            UserDataLoadable::Loading { content, .. } | UserDataLoadable::Ready { content } => {
                content
            }
        };
        let user_data_effects = match msg {
            Msg::Action(Action::Ctx(ActionCtx::SyncWithAPI)) => {
                // TODO
                Effects::none().unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::RetrieveFromStorage)) => {
                *self = UserDataLoadable::Loading {
                    request: UserDataRequest::Storage,
                    content: UserData::default(),
                };
                Effects::one(Box::new(
                    Env::get_storage(USER_DATA_KEY)
                        .map(|user_data| {
                            Msg::Internal(Internal::UserDataStorageResult(Box::new(user_data)))
                        })
                        .map_err(|error| {
                            Msg::Event(Event::ActionError(
                                Action::Ctx(ActionCtx::RetrieveFromStorage),
                                MsgError::from(error),
                            ))
                        }),
                ))
            }
            Msg::Action(Action::Ctx(ActionCtx::PersistToStorage)) => Effects::one(Box::new(
                Env::set_storage(USER_DATA_KEY, Some(user_data))
                    .map(|_| Msg::Event(Event::UserDataPersisted))
                    .map_err(|error| {
                        Msg::Event(Event::ActionError(
                            Action::Ctx(ActionCtx::PersistToStorage),
                            MsgError::from(error),
                        ))
                    }),
            ))
            .unchanged(),
            Msg::Action(Action::Ctx(ActionCtx::User(action_user))) => {
                let action_user = action_user.to_owned();
                match action_user {
                    ActionUser::Login { email, password } => {
                        let login_request = APIRequest::Login { email, password };
                        *self = UserDataLoadable::Loading {
                            request: UserDataRequest::API(login_request.to_owned()),
                            content: UserData::default(),
                        };
                        Effects::one(Box::new(
                            authenticate::<Env>(&login_request)
                                .and_then(|auth| {
                                    get_addons::<Env>(&auth.key).map(move |addons| UserData {
                                        auth: Some(auth),
                                        addons,
                                        settings: Settings::default(),
                                    })
                                })
                                .map(move |user_data| {
                                    Msg::Internal(Internal::UserDataRequestResponse(
                                        login_request,
                                        Box::new(user_data),
                                    ))
                                })
                                .map_err(move |error| {
                                    Msg::Event(Event::ActionError(
                                        Action::Ctx(ActionCtx::User(action_user)),
                                        error,
                                    ))
                                }),
                        ))
                    }
                    ActionUser::Register {
                        email,
                        password,
                        gdpr_consent,
                    } => {
                        let register_request = APIRequest::Register {
                            email,
                            password,
                            gdpr_consent,
                        };
                        *self = UserDataLoadable::Loading {
                            request: UserDataRequest::API(register_request.to_owned()),
                            content: UserData::default(),
                        };
                        Effects::one(Box::new(
                            authenticate::<Env>(&register_request)
                                .and_then(|auth| {
                                    get_addons::<Env>(&auth.key).map(move |addons| UserData {
                                        auth: Some(auth),
                                        addons,
                                        settings: Settings::default(),
                                    })
                                })
                                .map(move |user_data| {
                                    Msg::Internal(Internal::UserDataRequestResponse(
                                        register_request,
                                        Box::new(user_data),
                                    ))
                                })
                                .map_err(move |error| {
                                    Msg::Event(Event::ActionError(
                                        Action::Ctx(ActionCtx::User(action_user)),
                                        error,
                                    ))
                                }),
                        ))
                    }
                    ActionUser::Logout => match &user_data.auth {
                        Some(auth) => {
                            *self = UserDataLoadable::Ready {
                                content: UserData::default(),
                            };
                            let logout_request = APIRequest::Logout {
                                auth_key: auth.key.to_owned(),
                            };
                            Effects::msg(Msg::Event(Event::UserLoggedOut)).join(Effects::one(
                                Box::new(
                                    fetch_api::<Env, _>(&logout_request)
                                        .map(|_| Msg::Internal(Internal::NOOP))
                                        .map_err(move |error| {
                                            Msg::Event(Event::ActionError(
                                                Action::Ctx(ActionCtx::User(action_user)),
                                                error,
                                            ))
                                        }),
                                ),
                            ))
                        }
                        _ => {
                            *self = UserDataLoadable::Ready {
                                content: UserData::default(),
                            };
                            Effects::msg(Msg::Event(Event::UserLoggedOut))
                        }
                    },
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::Addons(action_addon))) => match action_addon {
                ActionAddons::Install(descriptor) => {
                    let position = user_data
                        .addons
                        .iter()
                        .position(|addon| addon.transport_url.eq(&descriptor.transport_url));
                    if let Some(position) = position {
                        user_data.addons.remove(position);
                    };
                    user_data.addons.push(descriptor.deref().to_owned());
                    Effects::msg(Msg::Event(Event::AddonInstalled))
                }
                ActionAddons::Uninstall { transport_url } => {
                    let position = user_data.addons.iter().position(|addon| {
                        addon.transport_url.eq(transport_url) && !addon.flags.protected
                    });
                    match position {
                        Some(position) => {
                            user_data.addons.remove(position);
                            Effects::msg(Msg::Event(Event::AddonUninstalled))
                        }
                        _ => Effects::none().unchanged(),
                    }
                }
            },
            Msg::Action(Action::Ctx(ActionCtx::Settings(action_settings))) => match action_settings
            {
                ActionSettings::Update(settings) => {
                    user_data.settings = settings.deref().to_owned();
                    Effects::msg(Msg::Event(Event::SettingsUpdated))
                }
            },
            Msg::Internal(Internal::UserDataStorageResult(user_data)) => match &self {
                UserDataLoadable::Loading { request, .. }
                    if request.eq(&UserDataRequest::Storage) =>
                {
                    *self = UserDataLoadable::Ready {
                        content: user_data.deref().to_owned().unwrap_or_default(),
                    };
                    Effects::msg(Msg::Event(Event::UserDataRetrivedFromStorage))
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserDataRequestResponse(api_request, user_data)) => match &self
            {
                UserDataLoadable::Loading { request, .. }
                    if request.eq(&UserDataRequest::API(api_request.to_owned())) =>
                {
                    *self = UserDataLoadable::Ready {
                        content: user_data.deref().to_owned(),
                    };
                    Effects::msg(Msg::Event(Event::UserAuthenticated))
                }
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        };
        if user_data_effects.has_changed {
            Effects::msg(Msg::Internal(Internal::UserDataChanged)).join(user_data_effects)
        } else {
            user_data_effects
        }
    }
}
