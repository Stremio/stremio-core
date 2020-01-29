use crate::constants::{OFFICIAL_ADDONS, STREAMING_SERVER_URL, USER_STORAGE_KEY};
use crate::state_types::models::common::{
    authenticate, delete_user_session, get_user_addons, set_user_addons, ModelError,
};
use crate::state_types::msg::{
    Action, ActionAddons, ActionAuth, ActionCtx, ActionLoad, ActionSettings, ActionUser, Event,
    Internal, Msg,
};
use crate::state_types::{Effects, Environment};
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, Auth};
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
pub struct User {
    pub auth: Option<Auth>,
    pub addons: Vec<Descriptor>,
    pub settings: Settings,
}

impl Default for User {
    fn default() -> Self {
        User {
            auth: None,
            addons: OFFICIAL_ADDONS.to_owned(),
            settings: Settings::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum UserRequest {
    Storage,
    API(APIRequest),
}

#[derive(Derivative, Clone, Debug, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(tag = "type")]
pub enum UserLoadable {
    Loading {
        request: UserRequest,
        content: User,
    },
    #[derivative(Default)]
    Ready {
        content: User,
    },
}

impl UserLoadable {
    pub fn update<Env: Environment + 'static>(&mut self, msg: &Msg) -> Effects {
        let user_effects = match msg {
            Msg::Action(Action::Load(ActionLoad::Ctx)) => {
                *self = UserLoadable::Loading {
                    request: UserRequest::Storage,
                    content: self.content().to_owned(),
                };
                Effects::one(Box::new(Env::get_storage(USER_STORAGE_KEY).then(
                    |result| {
                        Ok(Msg::Internal(Internal::UserStorageResult(
                            result.map_err(ModelError::from),
                        )))
                    },
                )))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Auth(action_auth)))) => {
                match action_auth {
                    ActionAuth::Login { email, password } => {
                        let request = APIRequest::Login {
                            email: email.to_owned(),
                            password: password.to_owned(),
                        };
                        *self = UserLoadable::Loading {
                            request: UserRequest::API(request.to_owned()),
                            content: self.content().to_owned(),
                        };
                        Effects::one(Box::new(
                            authenticate::<Env>(&request)
                                .and_then(|auth| {
                                    get_user_addons::<Env>(&auth.key)
                                        .map(move |addons| (auth, addons))
                                })
                                .then(move |result| {
                                    Ok(Msg::Internal(Internal::UserAuthResult(request, result)))
                                }),
                        ))
                        .unchanged()
                    }
                    ActionAuth::Register {
                        email,
                        password,
                        gdpr_consent,
                    } => {
                        let request = APIRequest::Register {
                            email: email.to_owned(),
                            password: password.to_owned(),
                            gdpr_consent: gdpr_consent.to_owned(),
                        };
                        *self = UserLoadable::Loading {
                            request: UserRequest::API(request.to_owned()),
                            content: self.content().to_owned(),
                        };
                        Effects::one(Box::new(
                            authenticate::<Env>(&request)
                                .and_then(|auth| {
                                    get_user_addons::<Env>(&auth.key)
                                        .map(move |addons| (auth, addons))
                                })
                                .then(move |result| {
                                    Ok(Msg::Internal(Internal::UserAuthResult(request, result)))
                                }),
                        ))
                        .unchanged()
                    }
                    ActionAuth::Logout => {
                        let session_effects = match self.auth() {
                            Some(auth) => Effects::one(Box::new(
                                delete_user_session::<Env>(&auth.key)
                                    .map(|_| Msg::Event(Event::UserSessionDeleted))
                                    .map_err(|error| Msg::Event(Event::Error(error))),
                            ))
                            .unchanged(),
                            _ => Effects::none().unchanged(),
                        };
                        *self = UserLoadable::Ready {
                            content: User::default(),
                        };
                        Effects::msg(Msg::Event(Event::UserLoggedOut)).join(session_effects)
                    }
                    ActionAuth::PushToAPI => Effects::none().unchanged(),
                    ActionAuth::PullFromAPI => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Addons(action_addons)))) => {
                match action_addons {
                    ActionAddons::Install(descriptor) => {
                        let user = self.content();
                        let addon_position = user
                            .addons
                            .iter()
                            .position(|addon| addon.transport_url.eq(&descriptor.transport_url));
                        if let Some(addon_position) = addon_position {
                            user.addons.remove(addon_position);
                        };
                        user.addons.push(descriptor.to_owned());
                        Effects::msg(Msg::Event(Event::AddonInstalled))
                    }
                    ActionAddons::Uninstall(transport_url) => {
                        let user = self.content();
                        let addon_position = user
                            .addons
                            .iter()
                            .position(|addon| addon.transport_url.eq(transport_url));
                        match addon_position {
                            Some(addon_position)
                                if !user.addons[addon_position].flags.protected =>
                            {
                                user.addons.remove(addon_position);
                                Effects::msg(Msg::Event(Event::AddonUninstalled))
                            }
                            _ => Effects::none().unchanged(),
                        }
                    }
                    ActionAddons::PushToAPI => match self.auth() {
                        Some(auth) => Effects::one(Box::new(
                            set_user_addons::<Env>(&auth.key, self.addons())
                                .map(|_| Msg::Event(Event::AddonsPushedToAPI))
                                .map_err(|error| Msg::Event(Event::Error(error))),
                        ))
                        .unchanged(),
                        _ => Effects::none().unchanged(),
                    },
                    ActionAddons::PullFromAPI => match self.auth() {
                        Some(auth) => {
                            let auth_key = auth.key.to_owned();
                            Effects::one(Box::new(get_user_addons::<Env>(&auth_key).then(
                                move |result| {
                                    Ok(Msg::Internal(Internal::UserAddonsResult(auth_key, result)))
                                },
                            )))
                            .unchanged()
                        }
                        _ => {
                            // TODO is there a better place for this piece of code ?
                            let next_addons = self
                                .addons()
                                .iter()
                                .map(|user_addon| {
                                    OFFICIAL_ADDONS
                                        .iter()
                                        .find(|Descriptor { manifest, .. }| {
                                            manifest.id.eq(&user_addon.manifest.id)
                                                && manifest.version.gt(&user_addon.manifest.version)
                                        })
                                        .map(|official_addon| Descriptor {
                                            transport_url: official_addon.transport_url.to_owned(),
                                            manifest: official_addon.manifest.to_owned(),
                                            flags: user_addon.flags.to_owned(),
                                        })
                                        .unwrap_or_else(|| user_addon.to_owned())
                                })
                                .collect();
                            let mut user = self.content();
                            if user.addons.ne(&next_addons) {
                                user.addons = next_addons;
                                Effects::none()
                            } else {
                                Effects::none().unchanged()
                            }
                        }
                    },
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Settings(action_settings)))) => {
                match action_settings {
                    ActionSettings::Update(settings) => {
                        let mut user = self.content();
                        if user.settings.ne(settings) {
                            user.settings = settings.to_owned();
                            Effects::msg(Msg::Event(Event::SettingsUpdated))
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                }
            }
            Msg::Internal(Internal::UserStorageResult(result)) => match &self {
                UserLoadable::Loading {
                    request: UserRequest::Storage,
                    ..
                } => match result {
                    Ok(user) => {
                        *self = UserLoadable::Ready {
                            content: user.to_owned().unwrap_or_default(),
                        };
                        Effects::msg(Msg::Event(Event::UserRetrievedFromStorage))
                    }
                    Err(error) => {
                        *self = UserLoadable::Ready {
                            content: self.content().to_owned(),
                        };
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserAuthResult(api_request, result)) => match &self {
                UserLoadable::Loading {
                    request: UserRequest::API(loading_api_request),
                    ..
                } if loading_api_request.eq(api_request) => match result {
                    Ok((auth, addons)) => {
                        *self = UserLoadable::Ready {
                            content: User {
                                auth: Some(auth.to_owned()),
                                addons: addons.to_owned(),
                                ..User::default()
                            },
                        };
                        Effects::msg(Msg::Event(Event::UserAuthenticated))
                    }
                    Err(error) => {
                        *self = UserLoadable::Ready {
                            content: self.content().to_owned(),
                        };
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserAddonsResult(auth_key, result))
                if self
                    .auth()
                    .as_ref()
                    .map(|auth| &auth.key)
                    .eq(&Some(auth_key)) =>
            {
                match result {
                    Ok(addons) => {
                        let mut user = self.content();
                        if user.addons.ne(addons) {
                            user.addons = addons.to_owned();
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI))
                        } else {
                            Effects::msg(Msg::Event(Event::AddonsPulledFromAPI)).unchanged()
                        }
                    }
                    Err(error) => {
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                }
            }
            _ => Effects::none().unchanged(),
        };
        if user_effects.has_changed {
            Effects::msg(Msg::Internal(Internal::UserChanged))
                .join(Effects::one(Box::new(
                    Env::set_storage(USER_STORAGE_KEY, Some(self.content()))
                        .map(|_| Msg::Event(Event::UserPersisted))
                        .map_err(|error| Msg::Event(Event::Error(ModelError::from(error)))),
                )))
                .join(user_effects)
        } else {
            user_effects
        }
    }
    pub fn content(&mut self) -> &mut User {
        match self {
            UserLoadable::Loading { content, .. } | UserLoadable::Ready { content } => content,
        }
    }
    pub fn auth(&self) -> &Option<Auth> {
        match &self {
            UserLoadable::Loading { content, .. } | UserLoadable::Ready { content } => {
                &content.auth
            }
        }
    }
    pub fn addons(&self) -> &Vec<Descriptor> {
        match &self {
            UserLoadable::Loading { content, .. } | UserLoadable::Ready { content } => {
                &content.addons
            }
        }
    }
    pub fn settings(&self) -> &Settings {
        match &self {
            UserLoadable::Loading { content, .. } | UserLoadable::Ready { content } => {
                &content.settings
            }
        }
    }
}
