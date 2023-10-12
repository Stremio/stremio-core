use stremio_core::{
    constants::{
        CATALOG_RESOURCE_NAME, CINEMETA_TOP_CATALOG_ID, CINEMETA_URL, STREAM_RESOURCE_NAME,
        WATCH_STATUS_RESOURCE_NAME,
    },
    types::{
        addon::{ExtraValue, ResourcePath, ResourceRequest},
        watch_status,
    },
};

fn main() {
    let _cinemeta_resource_request = ResourceRequest {
        base: CINEMETA_URL.to_owned(),
        path: ResourcePath {
            id: CINEMETA_TOP_CATALOG_ID.to_owned(),
            resource: CATALOG_RESOURCE_NAME.to_owned(),
            r#type: "movie".to_owned(),
            extra: vec![ExtraValue {
                name: "genre".to_owned(),
                value: "your-genre".to_owned(),
            }],
        },
    };

    let watch_status_request = watch_status::Request::Resume {
        // 1 hour mark in milliseconds
        current_time: (60_u64 * 60 * 1000),
        // 1.5 hours in milliseconds
        duration: (90_u64 * 60 * 1000),
    };

    let watch_status_resource_request = ResourceRequest {
        base: CINEMETA_URL.to_owned(),
        path: ResourcePath {
            id: "tt0944947".to_owned(),
            resource: WATCH_STATUS_RESOURCE_NAME.to_owned(),
            r#type: "series".to_owned(),
            extra: watch_status_request.into(),
        },
    };

    let watch_status_path = watch_status_resource_request.path.to_url_path();
    println!("watchStatus 'play' extraArgs: {}", watch_status_path);
    assert_eq!(
        "/watchStatus/series/tt0944947/action=resume&currentTime=3600000&duration=5400000.json",
        watch_status_path
    );

    let _stream_path = ResourcePath {
        resource: STREAM_RESOURCE_NAME.to_owned(),
        r#type: "serial".to_owned(),
        id: "tt0944947".to_owned(),
        extra: vec![],
    };
}
