use crate::types::addon::{Descriptor, Manifest};
use crate::types::profile::{Auth, GDPRConsent, Profile, User};
use chrono::prelude::TimeZone;
use chrono::Utc;
use semver::Version;
use url::Url;

#[test]
fn deserialize_profile() {
    let profile = Profile {
        auth: Some(Auth {
            key: "auth_key".to_owned(),
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
                    time: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                    from: "tests".to_owned(),
                },
            },
        }),
        addons: vec![Descriptor {
            manifest: Manifest {
                id: "id".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                contact_email: None,
                description: None,
                logo: None,
                background: None,
                types: vec![],
                resources: vec![],
                id_prefixes: None,
                catalogs: vec![],
                addon_catalogs: vec![],
                behavior_hints: Default::default(),
            },
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: Default::default(),
        }],
        ..Default::default()
    };
    let profile_json = "{\"auth\":{\"key\":\"auth_key\",\"user\":{\"_id\":\"user_id\",\"email\":\"user_email\",\"fbId\":null,\"avatar\":null,\"lastModified\":\"2020-01-01T00:00:00Z\",\"dateRegistered\":\"2020-01-01T00:00:00Z\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":true,\"time\":\"2020-01-01T00:00:00Z\",\"from\":\"tests\"}}},\"addons\":[{\"manifest\":{\"id\":\"id\",\"version\":\"0.0.1\",\"name\":\"name\",\"contactEmail\":null,\"description\":null,\"logo\":null,\"background\":null,\"types\":[],\"resources\":[],\"idPrefixes\":null,\"catalogs\":[],\"addonCatalogs\":[],\"behaviorHints\":{}},\"transportUrl\":\"https://transport_url/\",\"flags\":{\"official\":false,\"protected\":false}}],\"settings\":{\"interface_language\":\"eng\",\"streaming_server_url\":\"http://127.0.0.1:11470/\",\"binge_watching\":false,\"play_in_background\":true,\"play_in_external_player\":false,\"hardware_decoding\":false,\"subtitles_language\":\"eng\",\"subtitles_size\":100,\"subtitles_font\":\"Roboto\",\"subtitles_bold\":false,\"subtitles_offset\":5,\"subtitles_text_color\":\"#FFFFFFFF\",\"subtitles_background_color\":\"#00000000\",\"subtitles_outline_color\":\"#00000000\"}}";
    let profile_deserialize = serde_json::from_str(&profile_json).unwrap();
    assert_eq!(
        profile, profile_deserialize,
        "profile deserialized successfully"
    );
}
