use crate::types::addon::{
    DescriptorFlags, Manifest, ManifestBehaviorHints, ManifestPreview, ResourcePath,
};
use crate::types::api::{APIError, AuthRequest};
use crate::types::library::LibraryItemState;
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Settings, User};
use crate::types::resource::{
    MetaItem, MetaItemBehaviorHints, PosterShape, SeriesInfo, StreamBehaviorHints, StreamSource,
    Subtitles,
};
use crate::types::True;
use serde_test::Token;

pub trait DefaultTokens {
    fn default_tokens() -> Vec<Token>;
}

pub trait DefaultFlattenTokens {
    fn default_flatten_tokens() -> Vec<Token>;
}

impl DefaultTokens for LibraryItemState {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "LibraryItemState",
                len: 10,
            },
            Token::Str("lastWatched"),
            Token::None,
            Token::Str("timeWatched"),
            Token::U64(0),
            Token::Str("timeOffset"),
            Token::U64(0),
            Token::Str("overallTimeWatched"),
            Token::U64(0),
            Token::Str("timesWatched"),
            Token::U32(0),
            Token::Str("flaggedWatched"),
            Token::U32(0),
            Token::Str("duration"),
            Token::U64(0),
            Token::Str("video_id"),
            Token::None,
            Token::Str("watched"),
            Token::None,
            Token::Str("noNotif"),
            Token::Bool(false),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for Subtitles {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "Subtitles",
                len: 3,
            },
            Token::Str("id"),
            Token::Str(""),
            Token::Str("lang"),
            Token::Str(""),
            Token::Str("url"),
            Token::Str("protocol://host"),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for StreamBehaviorHints {
    fn default_tokens() -> Vec<Token> {
        vec![Token::Map { len: None }, Token::MapEnd]
    }
}

impl DefaultTokens for MetaItemBehaviorHints {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Map { len: None },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::Str("featuredVideoId"),
            Token::None,
            Token::Str("hasScheduledVideos"),
            Token::Bool(false),
            Token::MapEnd,
        ]
    }
}

impl DefaultTokens for PosterShape {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("poster"),
            Token::Unit,
        ]
    }
}

impl DefaultTokens for ManifestPreview {
    fn default_tokens() -> Vec<Token> {
        [
            vec![
                Token::Struct {
                    name: "ManifestPreview",
                    len: 8,
                },
                Token::Str("id"),
                Token::Str(""),
                Token::Str("version"),
                Token::Str("0.0.1"),
                Token::Str("name"),
                Token::Str(""),
                Token::Str("description"),
                Token::None,
                Token::Str("logo"),
                Token::None,
                Token::Str("background"),
                Token::None,
                Token::Str("types"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            ManifestBehaviorHints::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat()
    }
}

impl DefaultTokens for Manifest {
    fn default_tokens() -> Vec<Token> {
        [
            vec![
                Token::Struct {
                    name: "Manifest",
                    len: 13,
                },
                Token::Str("id"),
                Token::Str(""),
                Token::Str("version"),
                Token::Str("0.0.1"),
                Token::Str("name"),
                Token::Str(""),
                Token::Str("contactEmail"),
                Token::None,
                Token::Str("description"),
                Token::None,
                Token::Str("logo"),
                Token::None,
                Token::Str("background"),
                Token::None,
                Token::Str("types"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("resources"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("idPrefixes"),
                Token::None,
                Token::Str("catalogs"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("addonCatalogs"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            ManifestBehaviorHints::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat()
    }
}

impl DefaultTokens for ManifestBehaviorHints {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "ManifestBehaviorHints",
                len: 4,
            },
            Token::Str("adult"),
            Token::Bool(false),
            Token::Str("p2p"),
            Token::Bool(false),
            Token::Str("configurable"),
            Token::Bool(false),
            Token::Str("configurationRequired"),
            Token::Bool(false),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for DescriptorFlags {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "DescriptorFlags",
                len: 2,
            },
            Token::Str("official"),
            Token::Bool(false),
            Token::Str("protected"),
            Token::Bool(false),
            Token::StructEnd,
        ]
    }
}

impl DefaultFlattenTokens for SeriesInfo {
    fn default_flatten_tokens() -> Vec<Token> {
        vec![
            Token::Str("season"),
            Token::U32(0),
            Token::Str("episode"),
            Token::U32(0),
        ]
    }
}

impl DefaultTokens for MetaItem {
    fn default_tokens() -> Vec<Token> {
        [
            vec![
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str(""),
                Token::Str("type"),
                Token::Str(""),
                Token::Str("name"),
                Token::Str(""),
                Token::Str("poster"),
                Token::None,
                Token::Str("background"),
                Token::None,
                Token::Str("logo"),
                Token::None,
                Token::Str("description"),
                Token::None,
                Token::Str("releaseInfo"),
                Token::None,
                Token::Str("runtime"),
                Token::None,
                Token::Str("released"),
                Token::None,
                Token::Str("posterShape"),
            ],
            PosterShape::default_tokens(),
            vec![
                Token::Str("links"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("trailerStreams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            MetaItemBehaviorHints::default_tokens(),
            vec![
                Token::Str("videos"),
                Token::Some,
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::MapEnd,
            ],
        ]
        .concat()
    }
}

impl DefaultTokens for GDPRConsent {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "GDPRConsent",
                len: 4,
            },
            Token::Str("tos"),
            Token::Bool(false),
            Token::Str("privacy"),
            Token::Bool(false),
            Token::Str("marketing"),
            Token::Bool(false),
            Token::Str("from"),
            Token::None,
            Token::StructEnd,
        ]
    }
}

impl DefaultFlattenTokens for GDPRConsent {
    fn default_flatten_tokens() -> Vec<Token> {
        vec![
            Token::Str("tos"),
            Token::Bool(false),
            Token::Str("privacy"),
            Token::Bool(false),
            Token::Str("marketing"),
            Token::Bool(false),
            Token::Str("from"),
            Token::None,
        ]
    }
}

impl DefaultTokens for AuthKey {
    fn default_tokens() -> Vec<Token> {
        vec![Token::NewtypeStruct { name: "AuthKey" }, Token::Str("")]
    }
}

impl DefaultTokens for User {
    fn default_tokens() -> Vec<Token> {
        [
            vec![
                Token::Struct {
                    name: "User",
                    len: 9,
                },
                Token::Str("_id"),
                Token::Str(""),
                Token::Str("email"),
                Token::Str(""),
                Token::Str("fbId"),
                Token::None,
                Token::Str("avatar"),
                Token::None,
                Token::Str("lastModified"),
                Token::Str("1970-01-01T00:00:00Z"),
                Token::Str("dateRegistered"),
                Token::Str("1970-01-01T00:00:00Z"),
                Token::Str("trakt"),
                Token::None,
                Token::Str("premium_expire"),
                Token::None,
                Token::Str("gdpr_consent"),
            ],
            GDPRConsent::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat()
    }
}

impl DefaultTokens for Auth {
    fn default_tokens() -> Vec<Token> {
        [
            vec![
                Token::Struct {
                    name: "Auth",
                    len: 2,
                },
                Token::Str("key"),
            ],
            AuthKey::default_tokens(),
            vec![Token::Str("user")],
            User::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat()
    }
}

impl DefaultTokens for Settings {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "Settings",
                len: 29,
            },
            Token::Str("interfaceLanguage"),
            Token::Str("eng"),
            Token::Str("streamingServerUrl"),
            Token::Str("http://127.0.0.1:11470/"),
            Token::Str("playerType"),
            Token::None,
            Token::Str("bingeWatching"),
            Token::Bool(true),
            Token::Str("playInBackground"),
            Token::Bool(true),
            Token::Str("hardwareDecoding"),
            Token::Bool(true),
            Token::Str("frameRateMatchingStrategy"),
            Token::UnitVariant {
                name: "FrameRateMatchingStrategy",
                variant: "FrameRateOnly",
            },
            Token::Str("nextVideoNotificationDuration"),
            Token::U32(35000),
            Token::Str("audioPassthrough"),
            Token::Bool(false),
            Token::Str("audioLanguage"),
            Token::Some,
            Token::Str("eng"),
            Token::Str("secondaryAudioLanguage"),
            Token::None,
            Token::Str("subtitlesLanguage"),
            Token::Some,
            Token::Str("eng"),
            Token::Str("secondarySubtitlesLanguage"),
            Token::None,
            Token::Str("subtitlesSize"),
            Token::U8(100),
            Token::Str("subtitlesFont"),
            Token::Str("Roboto"),
            Token::Str("subtitlesBold"),
            Token::Bool(false),
            Token::Str("subtitlesOffset"),
            Token::U8(5),
            Token::Str("subtitlesTextColor"),
            Token::Str("#FFFFFFFF"),
            Token::Str("subtitlesBackgroundColor"),
            Token::Str("#00000000"),
            Token::Str("subtitlesOutlineColor"),
            Token::Str("#000000"),
            Token::Str("subtitlesOpacity"),
            Token::U8(100),
            Token::Str("escExitFullscreen"),
            Token::Bool(true),
            Token::Str("seekTimeDuration"),
            Token::U32(10000),
            Token::Str("seekShortTimeDuration"),
            Token::U32(3000),
            Token::Str("pauseOnMinimize"),
            Token::Bool(false),
            Token::Str("surroundSound"),
            Token::Bool(false),
            Token::Str("streamingServerWarningDismissed"),
            Token::None,
            Token::Str("serverInForeground"),
            Token::Bool(false),
            Token::Str("sendCrashReports"),
            Token::Bool(true),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for AuthRequest {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "AuthRequest",
                len: 5,
            },
            Token::Str("type"),
            Token::Str("Auth"),
            Token::Str("type"),
            Token::Str("Login"),
            Token::Str("email"),
            Token::Str(""),
            Token::Str("password"),
            Token::Str(""),
            Token::Str("facebook"),
            Token::Bool(false),
            Token::StructEnd,
        ]
    }
}

impl DefaultFlattenTokens for StreamSource {
    fn default_flatten_tokens() -> Vec<Token> {
        vec![Token::Str("ytId"), Token::Str("")]
    }
}

impl DefaultTokens for APIError {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "APIError",
                len: 2,
            },
            Token::Str("message"),
            Token::Str(""),
            Token::Str("code"),
            Token::U64(0),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for ResourcePath {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "ResourcePath",
                len: 4,
            },
            Token::Str("resource"),
            Token::Str(""),
            Token::Str("type"),
            Token::Str(""),
            Token::Str("id"),
            Token::Str(""),
            Token::Str("extra"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for True {
    fn default_tokens() -> Vec<Token> {
        vec![Token::Bool(true)]
    }
}
