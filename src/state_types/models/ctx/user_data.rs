use crate::constants::{OFFICIAL_ADDONS, USER_DATA_STORAGE_KEY};
use crate::state_types::models::common::{
    authenticate, delete_user_session, get_user_addons, set_user_addons,
};
use crate::state_types::msg::{
    Action, ActionAddons, ActionAuth, ActionCtx, ActionSettings, Event, Internal, Msg, MsgError,
};
use crate::state_types::{Effects, Environment};
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, Auth};
use derivative::Derivative;
use enclose::enclose;
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
#[serde(untagged)]
pub enum UserDataLoadable {
    Loading {
        #[serde(skip)]
        request: UserDataRequest,
        content: UserData,
    },
    #[derivative(Default)]
    Ready { content: UserData },
}

impl UserDataLoadable {
    pub fn update<Env: Environment + 'static>(&mut self, msg: &Msg) -> Effects {
        let user_data_effects = match msg {
            Msg::Action(Action::Ctx(action_ctx)) => {
                let action_ctx = action_ctx.to_owned();
                let action_error_msg = enclose!((action_ctx) move |error| Msg::Event(Event::CtxError {
                    action_ctx,
                    error
                }));
                match action_ctx {
                    ActionCtx::RetrieveFromStorage => {
                        *self = UserDataLoadable::Loading {
                            request: UserDataRequest::Storage,
                            content: self.user_data().to_owned(),
                        };
                        Effects::one(Box::new(
                            Env::get_storage(USER_DATA_STORAGE_KEY)
                                .map(|user_data| {
                                    Msg::Internal(Internal::UserDataStorageResponse(user_data))
                                })
                                .map_err(|error| action_error_msg(MsgError::from(error))),
                        ))
                        .unchanged()
                    }
                    ActionCtx::Auth(action_auth) => match action_auth {
                        ActionAuth::Login { email, password } => {
                            let request = APIRequest::Login { email, password };
                            *self = UserDataLoadable::Loading {
                                request: UserDataRequest::API(request.to_owned()),
                                content: self.user_data().to_owned(),
                            };
                            Effects::one(Box::new(
                                authenticate::<Env>(&request)
                                    .map(move |auth| {
                                        Msg::Internal(Internal::UserAuthResponse(request, auth))
                                    })
                                    .map_err(action_error_msg),
                            ))
                            .unchanged()
                        }
                        ActionAuth::Register {
                            email,
                            password,
                            gdpr_consent,
                        } => {
                            let request = APIRequest::Register {
                                email,
                                password,
                                gdpr_consent,
                            };
                            *self = UserDataLoadable::Loading {
                                request: UserDataRequest::API(request.to_owned()),
                                content: self.user_data().to_owned(),
                            };
                            Effects::one(Box::new(
                                authenticate::<Env>(&request)
                                    .map(move |auth| {
                                        Msg::Internal(Internal::UserAuthResponse(request, auth))
                                    })
                                    .map_err(action_error_msg),
                            ))
                            .unchanged()
                        }
                        ActionAuth::Logout => {
                            let delete_user_session_effects = match self.auth() {
                                Some(auth) => Effects::one(Box::new(
                                    delete_user_session::<Env>(&auth.key)
                                        .map(|_| Msg::Event(Event::UserSessionDeleted))
                                        .map_err(action_error_msg),
                                ))
                                .unchanged(),
                                _ => Effects::none().unchanged(),
                            };
                            *self = UserDataLoadable::Ready {
                                content: UserData::default(),
                            };
                            Effects::msg(Msg::Event(Event::UserLoggedOut))
                                .join(delete_user_session_effects)
                        }
                        ActionAuth::PushToAPI => Effects::none().unchanged(),
                        ActionAuth::PullFromAPI => Effects::none().unchanged(),
                    },
                    ActionCtx::Addons(action_addons) => match action_addons {
                        ActionAddons::PushToAPI => match self.auth() {
                            Some(auth) => Effects::one(Box::new(
                                set_user_addons::<Env>(&auth.key, self.addons())
                                    .map(|_| Msg::Event(Event::AddonsPushedToAPI))
                                    .map_err(action_error_msg),
                            ))
                            .unchanged(),
                            _ => Effects::none().unchanged(),
                        },
                        ActionAddons::PullFromAPI => match self.auth() {
                            Some(auth) => {
                                let auth_key = auth.key.to_owned();
                                Effects::one(Box::new(
                                    get_user_addons::<Env>(&auth_key)
                                        .map(move |addons| {
                                            Msg::Internal(Internal::UserAddonsResponse(
                                                auth_key, addons,
                                            ))
                                        })
                                        .map_err(action_error_msg),
                                ))
                                .unchanged()
                            }
                            _ => {
                                let next_addons = self
                                    .addons()
                                    .iter()
                                    .map(|user_addon| {
                                        OFFICIAL_ADDONS
                                            .iter()
                                            .find(|Descriptor { manifest, .. }| {
                                                manifest.id.eq(&user_addon.manifest.id)
                                                    && manifest
                                                        .version
                                                        .gt(&user_addon.manifest.version)
                                            })
                                            .map(|official_addon| Descriptor {
                                                transport_url: official_addon
                                                    .transport_url
                                                    .to_owned(),
                                                manifest: official_addon.manifest.to_owned(),
                                                flags: user_addon.flags.to_owned(),
                                            })
                                            .unwrap_or_else(|| user_addon.to_owned())
                                    })
                                    .collect();
                                let mut user_data = self.user_data();
                                if user_data.addons.ne(&next_addons) {
                                    user_data.addons = next_addons;
                                    Effects::none()
                                } else {
                                    Effects::none().unchanged()
                                }
                            }
                        },
                        ActionAddons::Install(descriptor) => {
                            let user_data = self.user_data();
                            let addon_position = user_data.addons.iter().position(|addon| {
                                addon.transport_url.eq(&descriptor.transport_url)
                            });
                            if let Some(addon_position) = addon_position {
                                user_data.addons.remove(addon_position);
                            };
                            user_data.addons.push(descriptor);
                            Effects::msg(Msg::Event(Event::AddonInstalled))
                        }
                        ActionAddons::Uninstall { transport_url } => {
                            let user_data = self.user_data();
                            let addon_position = user_data.addons.iter().position(|addon| {
                                addon.transport_url.eq(&transport_url) && !addon.flags.protected
                            });
                            match addon_position {
                                Some(addon_position) => {
                                    user_data.addons.remove(addon_position);
                                    Effects::msg(Msg::Event(Event::AddonUninstalled))
                                }
                                _ => Effects::none().unchanged(),
                            }
                        }
                    },
                    ActionCtx::Settings(ActionSettings::Update(settings)) => {
                        let mut user_data = self.user_data();
                        user_data.settings = settings;
                        Effects::msg(Msg::Event(Event::SettingsUpdated))
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::UserDataStorageResponse(user_data)) => match &self {
                UserDataLoadable::Loading {
                    request: UserDataRequest::Storage,
                    ..
                } => {
                    *self = UserDataLoadable::Ready {
                        content: user_data.to_owned().unwrap_or_default(),
                    };
                    Effects::msg(Msg::Event(Event::UserDataRetrievedFromStorage))
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserAuthResponse(api_request, auth)) => match &self {
                UserDataLoadable::Loading {
                    request: UserDataRequest::API(loading_api_request),
                    ..
                } if loading_api_request.eq(api_request) => {
                    *self = UserDataLoadable::Ready {
                        content: UserData {
                            auth: Some(auth.to_owned()),
                            ..Default::default()
                        },
                    };
                    Effects::msg(Msg::Event(Event::UserAuthenticated)).join(Effects::msg(
                        Msg::Action(Action::Ctx(ActionCtx::Addons(ActionAddons::PullFromAPI))),
                    ))
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserAddonsResponse(auth_key, addons))
                if self
                    .auth()
                    .as_ref()
                    .map(|auth| &auth.key)
                    .eq(&Some(auth_key)) =>
            {
                let mut user_data = self.user_data();
                if user_data.addons.ne(addons) {
                    user_data.addons = addons.to_owned();
                    Effects::msg(Msg::Event(Event::AddonsPulledFromAPI))
                } else {
                    Effects::msg(Msg::Event(Event::AddonsPulledFromAPI)).unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        };
        if user_data_effects.has_changed {
            Effects::msg(Msg::Internal(Internal::UserDataChanged))
                .join(Effects::one(Box::new(
                    Env::set_storage(USER_DATA_STORAGE_KEY, Some(self.user_data()))
                        .map(|_| Msg::Event(Event::UserDataPersisted))
                        .map_err(|error| {
                            Msg::Event(Event::Error {
                                error: MsgError::from(error),
                            })
                        }),
                )))
                .join(user_data_effects)
        } else {
            user_data_effects
        }
    }
    pub fn auth(&self) -> &Option<Auth> {
        match &self {
            UserDataLoadable::Loading { content, .. } | UserDataLoadable::Ready { content } => {
                &content.auth
            }
        }
    }
    pub fn addons(&self) -> &Vec<Descriptor> {
        match &self {
            UserDataLoadable::Loading { content, .. } | UserDataLoadable::Ready { content } => {
                &content.addons
            }
        }
    }
    pub fn settings(&self) -> &Settings {
        match &self {
            UserDataLoadable::Loading { content, .. } | UserDataLoadable::Ready { content } => {
                &content.settings
            }
        }
    }
    fn user_data(&mut self) -> &mut UserData {
        match self {
            UserDataLoadable::Loading { content, .. } | UserDataLoadable::Ready { content } => {
                content
            }
        }
    }
}
