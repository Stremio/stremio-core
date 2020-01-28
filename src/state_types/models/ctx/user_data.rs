use crate::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, OFFICIAL_ADDONS, USER_DATA_STORAGE_KEY,
};
use crate::state_types::models::common::{
    authenticate, delete_user_session, get_user_addons, set_user_addons, ModelError,
};
use crate::state_types::models::ctx::library::{lib_pull, LibraryLoadable, LibraryRequest};
use crate::state_types::msg::{
    Action, ActionAddons, ActionAuth, ActionCtx, ActionLoad, ActionSettings, ActionUser, Event,
    Internal, Msg,
};
use crate::state_types::{Effects, Environment};
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, Auth};
use crate::types::{LibBucket, UID};
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

// TODO: find a better name for this that does not contains "Data".
// Simply User is maybe not good too because there is another struct called User
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
    pub fn update<Env: Environment + 'static>(
        &mut self,
        library: &mut LibraryLoadable,
        msg: &Msg,
    ) -> Effects {
        let user_data_effects = match msg {
            Msg::Action(Action::Load(ActionLoad::Ctx)) => {
                *self = UserDataLoadable::Loading {
                    request: UserDataRequest::Storage,
                    content: self.user_data().to_owned(),
                };
                Effects::one(Box::new(Env::get_storage(USER_DATA_STORAGE_KEY).then(
                    |result| {
                        Ok(Msg::Internal(Internal::UserDataStorageResult(
                            result.map_err(ModelError::from),
                        )))
                    },
                )))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Auth(ActionAuth::Login {
                email,
                password,
            })))) => {
                let request = APIRequest::Login {
                    email: email.to_owned(),
                    password: password.to_owned(),
                };
                *self = UserDataLoadable::Loading {
                    request: UserDataRequest::API(request.to_owned()),
                    content: self.user_data().to_owned(),
                };
                Effects::one(Box::new(
                    authenticate::<Env>(&request)
                        .and_then(|auth| {
                            get_user_addons::<Env>(&auth.key).map(move |addons| (auth, addons))
                        })
                        .then(move |result| {
                            Ok(Msg::Internal(Internal::UserAuthResult(request, result)))
                        }),
                ))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Auth(ActionAuth::Register {
                email,
                password,
                gdpr_consent,
            })))) => {
                let request = APIRequest::Register {
                    email: email.to_owned(),
                    password: password.to_owned(),
                    gdpr_consent: gdpr_consent.to_owned(),
                };
                *self = UserDataLoadable::Loading {
                    request: UserDataRequest::API(request.to_owned()),
                    content: self.user_data().to_owned(),
                };
                Effects::one(Box::new(
                    authenticate::<Env>(&request)
                        .and_then(|auth| {
                            get_user_addons::<Env>(&auth.key).map(move |addons| (auth, addons))
                        })
                        .then(move |result| {
                            Ok(Msg::Internal(Internal::UserAuthResult(request, result)))
                        }),
                ))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Auth(ActionAuth::Logout)))) => {
                let logout_effects = match self.auth() {
                    Some(auth) => Effects::one(Box::new(
                        delete_user_session::<Env>(&auth.key)
                            .map(|_| Msg::Event(Event::UserSessionDeleted))
                            .map_err(|error| Msg::Event(Event::Error(error))),
                    ))
                    .unchanged(),
                    _ => Effects::none().unchanged(),
                };
                *self = UserDataLoadable::Ready {
                    content: UserData::default(),
                };
                *library = LibraryLoadable::Ready(LibBucket::default());
                Effects::msg(Msg::Event(Event::UserLoggedOut))
                    .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                    .join(logout_effects)
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Auth(ActionAuth::PushToAPI)))) => {
                Effects::none().unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Auth(
                ActionAuth::PullFromAPI,
            )))) => Effects::none().unchanged(),
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Addons(
                ActionAddons::Install(descriptor),
            )))) => {
                let user_data = self.user_data();
                let addon_position = user_data
                    .addons
                    .iter()
                    .position(|addon| addon.transport_url.eq(&descriptor.transport_url));
                if let Some(addon_position) = addon_position {
                    user_data.addons.remove(addon_position);
                };
                user_data.addons.push(descriptor.to_owned());
                Effects::msg(Msg::Event(Event::AddonInstalled))
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Addons(
                ActionAddons::Uninstall(transport_url),
            )))) => {
                let user_data = self.user_data();
                let addon_position = user_data
                    .addons
                    .iter()
                    .position(|addon| addon.transport_url.eq(transport_url));
                match addon_position {
                    Some(addon_position) if !user_data.addons[addon_position].flags.protected => {
                        user_data.addons.remove(addon_position);
                        Effects::msg(Msg::Event(Event::AddonUninstalled))
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Addons(
                ActionAddons::PushToAPI,
            )))) => match self.auth() {
                Some(auth) => Effects::one(Box::new(
                    set_user_addons::<Env>(&auth.key, self.addons())
                        .map(|_| Msg::Event(Event::AddonsPushedToAPI))
                        .map_err(|error| Msg::Event(Event::Error(error))),
                ))
                .unchanged(),
                _ => Effects::none().unchanged(),
            },
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Addons(
                ActionAddons::PullFromAPI,
            )))) => {
                match self.auth() {
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
                        let mut user_data = self.user_data();
                        if user_data.addons.ne(&next_addons) {
                            user_data.addons = next_addons;
                            Effects::none()
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::User(ActionUser::Settings(
                ActionSettings::Update(settings),
            )))) => {
                let mut user_data = self.user_data();
                if user_data.settings.ne(settings) {
                    user_data.settings = settings.to_owned();
                    Effects::msg(Msg::Event(Event::SettingsUpdated))
                } else {
                    Effects::none().unchanged()
                }
            }
            Msg::Internal(Internal::UserDataStorageResult(result)) => match &self {
                UserDataLoadable::Loading {
                    request: UserDataRequest::Storage,
                    ..
                } => match result {
                    Ok(user_data) => {
                        *self = UserDataLoadable::Ready {
                            content: user_data.to_owned().unwrap_or_default(),
                        };
                        *library = LibraryLoadable::Loading(
                            UID(self.auth().as_ref().map(|auth| auth.user.id.to_owned())),
                            LibraryRequest::Storage,
                        );
                        Effects::msg(Msg::Event(Event::UserDataRetrievedFromStorage))
                            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                            .join(Effects::one(Box::new(
                                Env::get_storage(LIBRARY_RECENT_STORAGE_KEY)
                                    .join(Env::get_storage(LIBRARY_STORAGE_KEY))
                                    .then(|result| {
                                        Ok(Msg::Internal(Internal::LibraryStorageResult(
                                            result.map_err(ModelError::from),
                                        )))
                                    }),
                            )))
                    }
                    Err(error) => {
                        *self = UserDataLoadable::Ready {
                            content: self.user_data().to_owned(),
                        };
                        Effects::msg(Msg::Event(Event::Error(error.to_owned()))).unchanged()
                    }
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::UserAuthResult(api_request, result)) => match &self {
                UserDataLoadable::Loading {
                    request: UserDataRequest::API(loading_api_request),
                    ..
                } if loading_api_request.eq(api_request) => match result {
                    Ok((auth, addons)) => {
                        let auth = auth.to_owned();
                        let uid = UID(Some(auth.user.id.to_owned()));
                        *self = UserDataLoadable::Ready {
                            content: UserData {
                                auth: Some(auth.to_owned()),
                                addons: addons.to_owned(),
                                ..UserData::default()
                            },
                        };
                        *library = LibraryLoadable::Loading(uid.to_owned(), LibraryRequest::API);
                        Effects::msg(Msg::Event(Event::UserAuthenticated))
                            .join(Effects::msg(Msg::Internal(Internal::LibraryChanged)))
                            .join(Effects::one(Box::new(lib_pull::<Env>(&auth).then(
                                move |result| {
                                    Ok(Msg::Internal(Internal::LibraryAPIResult(uid, result)))
                                },
                            ))))
                    }
                    Err(error) => {
                        *self = UserDataLoadable::Ready {
                            content: self.user_data().to_owned(),
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
                        let mut user_data = self.user_data();
                        if user_data.addons.ne(addons) {
                            user_data.addons = addons.to_owned();
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
        if user_data_effects.has_changed {
            Effects::msg(Msg::Internal(Internal::UserDataChanged))
                .join(Effects::one(Box::new(
                    Env::set_storage(USER_DATA_STORAGE_KEY, Some(self.user_data()))
                        .map(|_| Msg::Event(Event::UserDataPersisted))
                        .map_err(|error| Msg::Event(Event::Error(ModelError::from(error)))),
                )))
                .join(user_data_effects)
        } else {
            user_data_effects
        }
    }
    pub fn user_data(&mut self) -> &mut UserData {
        match self {
            UserDataLoadable::Loading { content, .. } | UserDataLoadable::Ready { content } => {
                content
            }
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
}
