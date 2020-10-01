use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, Settings, User};
use chrono::prelude::TimeZone;
use chrono::Utc;
use url::Url;

#[test]
fn deserialize_profile() {
    let profile = Profile {
        addons: vec![],
        settings: Settings {
            interface_language: "interface_language".to_owned(),
            streaming_server_url: Url::parse("https://streaming_server_url").unwrap(),
            binge_watching: false,
            play_in_background: true,
            play_in_external_player: false,
            hardware_decoding: false,
            subtitles_language: "subtitles_language".to_owned(),
            subtitles_size: 100,
            subtitles_font: "subtitles_font".to_owned(),
            subtitles_bold: false,
            subtitles_offset: 5,
            subtitles_text_color: "subtitles_text_color".to_owned(),
            subtitles_background_color: "subtitles_background_color".to_owned(),
            subtitles_outline_color: "subtitles_outline_color".to_owned(),
        },
        ..Default::default()
    };
    let profile_json = r##"
    {
        "auth": null,
        "addons": [],
        "settings": {
            "interface_language": "interface_language",
            "streaming_server_url": "https://streaming_server_url",
            "binge_watching": false,
            "play_in_background": true,
            "play_in_external_player": false,
            "hardware_decoding": false,
            "subtitles_language": "subtitles_language",
            "subtitles_size": 100,
            "subtitles_font": "subtitles_font",
            "subtitles_bold": false,
            "subtitles_offset": 5,
            "subtitles_text_color": "subtitles_text_color",
            "subtitles_background_color": "subtitles_background_color",
            "subtitles_outline_color": "subtitles_outline_color"
        }
    }
    "##;
    let profile_deserialize = serde_json::from_str(&profile_json).unwrap();
    assert_eq!(
        profile, profile_deserialize,
        "profile deserialized successfully"
    );
}

#[test]
fn deserialize_profile_with_user() {
    let profile = Profile {
        auth: Some(Auth {
            key: AuthKey("auth_key".to_owned()),
            user: User {
                id: "user_id".to_owned(),
                email: "user_email".to_owned(),
                fb_id: None,
                avatar: None,
                last_modified: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                date_registered: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                gdpr_consent: GDPRConsent {
                    tos: true,
                    privacy: true,
                    marketing: true,
                    from: "tests".to_owned(),
                },
            },
        }),
        addons: vec![],
        settings: Settings {
            interface_language: "interface_language".to_owned(),
            streaming_server_url: Url::parse("https://streaming_server_url").unwrap(),
            binge_watching: false,
            play_in_background: true,
            play_in_external_player: false,
            hardware_decoding: false,
            subtitles_language: "subtitles_language".to_owned(),
            subtitles_size: 100,
            subtitles_font: "subtitles_font".to_owned(),
            subtitles_bold: false,
            subtitles_offset: 5,
            subtitles_text_color: "subtitles_text_color".to_owned(),
            subtitles_background_color: "subtitles_background_color".to_owned(),
            subtitles_outline_color: "subtitles_outline_color".to_owned(),
        },
    };
    let profile_json = r##"
    {
        "auth": {
            "key": "auth_key",
            "user": {
                "_id": "user_id",
                "email": "user_email",
                "fbId": null,
                "avatar": null,
                "lastModified": "2020-01-01T00:00:00Z",
                "dateRegistered": "2020-01-01T00:00:00Z",
                "gdpr_consent": {
                    "tos": true,
                    "privacy": true,
                    "marketing": true,
                    "from": "tests"
                }
            }
        },
        "addons": [],
        "settings": {
            "interface_language": "interface_language",
            "streaming_server_url": "https://streaming_server_url",
            "binge_watching": false,
            "play_in_background": true,
            "play_in_external_player": false,
            "hardware_decoding": false,
            "subtitles_language": "subtitles_language",
            "subtitles_size": 100,
            "subtitles_font": "subtitles_font",
            "subtitles_bold": false,
            "subtitles_offset": 5,
            "subtitles_text_color": "subtitles_text_color",
            "subtitles_background_color": "subtitles_background_color",
            "subtitles_outline_color": "subtitles_outline_color"
        }
    }
    "##;
    let profile_deserialize = serde_json::from_str(&profile_json).unwrap();
    assert_eq!(
        profile, profile_deserialize,
        "profile deserialized successfully"
    );
}
