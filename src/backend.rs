use std::f32::consts::E;
use std::marker::PhantomData;

use crate::constants::{
    DISMISSED_EVENTS_STORAGE_KEY, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
    NOTIFICATIONS_STORAGE_KEY, PROFILE_STORAGE_KEY, SEARCH_HISTORY_STORAGE_KEY,
    STREAMING_SERVER_URLS_STORAGE_KEY, STREAMS_STORAGE_KEY,
};
use crate::models::common::Loadable;
use crate::runtime::{Env, EnvError, Runtime, RuntimeEvent};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::Profile;
use crate::types::resource::Stream;
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;

// use crate::env::{E, AppleEvent, APPLICATION_VERSION, SYSTEM_LANGUAGE};
// use crate::model::{AppleModel, Application};
// use crate_protobuf::{
//     bridge::{FromProtobuf, ToProtobuf},
//     protobuf::stremio::core::runtime::{self, Field},
// };

pub struct Backend<E: Env + 'static> {
    _env: PhantomData<E>,
}

impl<E> Backend<E>
where
    E: Env + 'static,
{
    pub fn run(&self) {
        let init_result = E::exec_sync(E::init());

        // Set the device name only once on initialization!
        // APPLICATION_VERSION
        //     .set({
        //         let cstr = unsafe { CStr::from_ptr(application.version) };
        //         // Get copy-on-write Cow<'_, str>, then guarantee a freshly-owned String allocation
        //         String::from_utf8_lossy(cstr.to_bytes()).to_string()
        //     })
        //     .expect("Application version should be set only once!");

        // // Set the device name only once on initialization!
        // SYSTEM_LANGUAGE
        //     .set({
        //         let cstr = unsafe { CStr::from_ptr(application.system_language) };
        //         // Get copy-on-write Cow<'_, str>, then guarantee a freshly-owned String allocation
        //         String::from_utf8_lossy(cstr.to_bytes()).to_string()
        //     })
        //     .expect("System Language should be set only once!");

        // // Set the device name only once on initialization!
        // DEVICE_NAME
        //     .set(unsafe { &*device_info }.to_string())
        //     .expect("Device name should be set only once!");

        match init_result {
            Ok(_) => {
                let storage_result = E::exec_sync(async {
                    futures::try_join!(
                        E::get_storage::<Profile>(PROFILE_STORAGE_KEY),
                        E::get_storage::<LibraryBucket>(LIBRARY_RECENT_STORAGE_KEY),
                        E::get_storage::<LibraryBucket>(LIBRARY_STORAGE_KEY),
                        E::get_storage::<StreamsBucket>(STREAMS_STORAGE_KEY),
                        E::get_storage::<ServerUrlsBucket>(
                            STREAMING_SERVER_URLS_STORAGE_KEY
                        ),
                        E::get_storage::<NotificationsBucket>(NOTIFICATIONS_STORAGE_KEY),
                        E::get_storage::<SearchHistoryBucket>(SEARCH_HISTORY_STORAGE_KEY),
                        E::get_storage::<DismissedEventsBucket>(
                            DISMISSED_EVENTS_STORAGE_KEY
                        ),
                    )
                });
                match storage_result {
                    Ok((
                        profile,
                        recent_bucket,
                        other_bucket,
                        streams,
                        server_urls_bucket,
                        notifications,
                        search_history,
                        dismissed_events,
                    )) => {
                        let profile = profile.unwrap_or_default();
                        let mut library = LibraryBucket::new(profile.uid(), vec![]);
                        if let Some(recent_bucket) = recent_bucket {
                            library.merge_bucket(recent_bucket);
                        };
                        if let Some(other_bucket) = other_bucket {
                            library.merge_bucket(other_bucket);
                        };
                        let streams = streams.unwrap_or(StreamsBucket::new(profile.uid()));
                        let server_urls_bucket = server_urls_bucket
                            .unwrap_or(ServerUrlsBucket::new::<E>(profile.uid()));
                        let notifications = notifications.unwrap_or(NotificationsBucket::new::<
                            E,
                        >(
                            profile.uid(), vec![]
                        ));
                        let search_history =
                            search_history.unwrap_or(SearchHistoryBucket::new(profile.uid()));
                        let dismissed_events =
                            dismissed_events.unwrap_or(DismissedEventsBucket::new(profile.uid()));
                        let (model, effects) = AppleModel::new(
                            &application,
                            profile,
                            library,
                            streams,
                            server_urls_bucket,
                            notifications,
                            search_history,
                            dismissed_events,
                        );
                        let (runtime, rx) = Runtime::<E, _>::new(
                            model,
                            effects.into_iter().collect::<Vec<_>>(),
                            1000,
                        );
                        let crate_class = class!(Core);
                        E::exec_concurrent(rx.for_each(move |event| {
                            if let RuntimeEvent::CoreEvent(event) = &event {
                                E::exec_concurrent(enclose!((event) async move {
                                    let runtime = RUNTIME.read().expect("runtime read failed");
                                    let runtime = runtime
                                        .as_ref()
                                        .expect("runtime is not ready")
                                        .as_ref()
                                        .expect("runtime is not ready");
                                    let model = runtime.model().expect("model read failed");
                                    E::emit_to_analytics(
                                        &AppleEvent::CoreEvent(event.to_owned()),
                                        &model,
                                        &Application::get_path(),
                                    );
                                }));
                            };
                            let eventbytes = &NSData::with_bytes(
                                &event.to_protobuf::<E>(&()).encode_to_vec(),
                            );
                            let _: () = unsafe {
                                msg_send![crate_class, onRuntimeEvent: eventbytes.as_ref()]
                            };
                            future::ready(())
                        }));
                        *RUNTIME.write().expect("RUNTIME write failed") =
                            Some(Loadable::Ready(runtime));
                        Nil as *mut NSObject
                    }
                    Err(error) => {
                        *RUNTIME.write().expect("RUNTIME write failed") =
                            Some(Loadable::Err(error.to_owned()));
                        let result_bytes = error.to_protobuf::<E>(&()).encode_to_vec();
                        Retained::into_raw(NSData::with_bytes(result_bytes.as_ref()))
                            as *mut NSObject
                    }
                }
            }
            Err(error) => {
                *RUNTIME.write().expect("RUNTIME write failed") =
                    Some(Loadable::Err(error.to_owned()));
                let result_bytes = error.to_protobuf::<E>(&()).encode_to_vec();
                Retained::into_raw(NSData::with_bytes(result_bytes.as_ref())) as *mut NSObject
            }
        }
    }
}
