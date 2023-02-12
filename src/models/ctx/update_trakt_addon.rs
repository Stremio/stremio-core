use crate::constants::URI_COMPONENT_ENCODE_SET;
use crate::models::common::{
    descriptor_update, eq_update, DescriptorAction, DescriptorLoadable, Loadable,
};
use crate::models::ctx::{CtxError, CtxStatus, OtherError};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effects, Env};
use crate::types::profile::Profile;
use percent_encoding::utf8_percent_encode;
use url::Url;

pub fn update_trakt_addon<E: Env + 'static>(
    trakt_addon: &mut Option<DescriptorLoadable>,
    profile: &Profile,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::Logout)) | Msg::Internal(Internal::Logout) => {
            eq_update(trakt_addon, None)
        }
        Msg::Action(Action::Ctx(ActionCtx::InstallTraktAddon)) => {
            let uid = profile.uid();
            match uid {
                Some(uid) => descriptor_update::<E>(
                    trakt_addon,
                    DescriptorAction::DescriptorRequested {
                        transport_url: &Url::parse(&format!(
                            "https://www.strem.io/trakt/addon/{}/manifest.json",
                            utf8_percent_encode(&uid, URI_COMPONENT_ENCODE_SET)
                        ))
                        .expect("Failed to parse trakt addon transport url"),
                    },
                ),
                _ => Effects::msg(Msg::Event(Event::Error {
                    error: CtxError::from(OtherError::UserNotLoggedIn),
                    source: Box::new(Event::TraktAddonFetched { uid }),
                }))
                .unchanged(),
            }
        }
        Msg::Internal(Internal::ManifestRequestResult(transport_url, result)) => {
            let trakt_addon_effects = descriptor_update::<E>(
                trakt_addon,
                DescriptorAction::ManifestRequestResult {
                    transport_url,
                    result,
                },
            );
            let trakt_addon_events_effects = match trakt_addon {
                Some(DescriptorLoadable {
                    content: Loadable::Ready(addon),
                    ..
                }) if trakt_addon_effects.has_changed => Effects::msgs(vec![
                    Msg::Event(Event::TraktAddonFetched { uid: profile.uid() }),
                    Msg::Internal(Internal::InstallAddon(addon.to_owned())),
                ])
                .unchanged(),
                Some(DescriptorLoadable {
                    content: Loadable::Err(error),
                    ..
                }) if trakt_addon_effects.has_changed => Effects::msg(Msg::Event(Event::Error {
                    error: CtxError::Env(error.to_owned()),
                    source: Box::new(Event::TraktAddonFetched { uid: profile.uid() }),
                }))
                .unchanged(),
                _ => Effects::none().unchanged(),
            };
            *trakt_addon = None;
            trakt_addon_effects.join(trakt_addon_events_effects)
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok(_))
                if loading_auth_request == auth_request =>
            {
                eq_update(trakt_addon, None)
            }
            _ => Effects::none().unchanged(),
        },
        _ => Effects::none().unchanged(),
    }
}
