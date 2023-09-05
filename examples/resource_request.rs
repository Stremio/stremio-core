use stremio_core::{
    constants::{
        CATALOG_RESOURCE_NAME, CINEMETA_TOP_CATALOG_ID, CINEMETA_URL, STREAM_RESOURCE_NAME,
        WATCH_STATUS_RESOURCE_NAME,
    },
    types::addon::{ExtraValue, ResourcePath, ResourceRequest},
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
    let watch_status_resource_request = ResourceRequest {
        base: CINEMETA_URL.to_owned(),
        path: ResourcePath {
            id: "tt0944947".to_owned(),
            resource: WATCH_STATUS_RESOURCE_NAME.to_owned(),
            r#type: "series".to_owned(),
            extra: vec![
                ExtraValue {
                    name: "action".to_owned(),
                    value: "play".to_owned(),
                },
                ExtraValue {
                    name: "currentTime".to_owned(),
                    // 1 hour mark in milliseconds
                    value: (60_u64 * 60 * 1000).to_string(),
                },
                ExtraValue {
                    name: "duration".to_owned(),
                    // 1.5 hours in milliseconds
                    value: (90_u64 * 60 * 1000).to_string(),
                },
            ],
        },
    };

    let watch_status_path = watch_status_resource_request.path.to_url_path();
    println!("{}", watch_status_path);
    assert_eq!(
        // start,end,pause,
        "/watchStatus/series/tt0944947/action=play&currentTime=3600000&duration=5400000.json",
        watch_status_path
    );

    let _stream_path = ResourcePath {
        resource: STREAM_RESOURCE_NAME.to_owned(),
        r#type: "serial".to_owned(),
        id: "tt0944947".to_owned(),
        extra: vec![],
    };
}
