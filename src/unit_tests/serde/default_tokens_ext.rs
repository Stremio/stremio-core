use crate::types::addon::{DescriptorFlags, Manifest, ManifestPreview};
use crate::types::api::{AuthRequest, GDPRConsentRequest};
use crate::types::library::{LibraryItemBehaviorHints, LibraryItemState};
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Settings, User};
use crate::types::resource::{MetaItem, MetaItemBehaviorHints, PosterShape};
use serde_test::Token;

pub trait DefaultTokens {
    fn default_tokens() -> Vec<Token>;
}

impl DefaultTokens for LibraryItemState {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "LibraryItemState",
                len: 11,
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
            Token::Str("lastVidReleased"),
            Token::None,
            Token::Str("noNotif"),
            Token::Bool(false),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for LibraryItemBehaviorHints {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "LibraryItemBehaviorHints",
                len: 1,
            },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for MetaItemBehaviorHints {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "MetaItemBehaviorHints",
                len: 3,
            },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::Str("featuredVideoId"),
            Token::None,
            Token::Str("hasScheduledVideos"),
            Token::Bool(false),
            Token::StructEnd,
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
        vec![
            Token::Struct {
                name: "ManifestPreview",
                len: 7,
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
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for Manifest {
    fn default_tokens() -> Vec<Token> {
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
            Token::Map { len: Some(0) },
            Token::MapEnd,
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

impl DefaultTokens for MetaItem {
    fn default_tokens() -> Vec<Token> {
        [
            vec![
                Token::Struct {
                    name: "MetaItem",
                    len: 16,
                },
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
                Token::Str("popularity"),
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
                Token::Str("videos"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("links"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("trailerStreams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            MetaItemBehaviorHints::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat()
    }
}

impl DefaultTokens for GDPRConsent {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Str("tos"),
            Token::Bool(false),
            Token::Str("privacy"),
            Token::Bool(false),
            Token::Str("marketing"),
            Token::Bool(false),
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
                    len: 7,
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
                Token::Str("gdpr_consent"),
                Token::Struct {
                    name: "GDPRConsent",
                    len: 3,
                },
            ],
            GDPRConsent::default_tokens(),
            vec![Token::StructEnd, Token::StructEnd],
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
                len: 14,
            },
            Token::Str("interfaceLanguage"),
            Token::Str("eng"),
            Token::Str("streamingServerUrl"),
            Token::Str("http://127.0.0.1:11470/"),
            Token::Str("bingeWatching"),
            Token::Bool(false),
            Token::Str("playInBackground"),
            Token::Bool(true),
            Token::Str("playInExternalPlayer"),
            Token::Bool(false),
            Token::Str("hardwareDecoding"),
            Token::Bool(false),
            Token::Str("subtitlesLanguage"),
            Token::Str("eng"),
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
            Token::Str("#00000000"),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for GDPRConsentRequest {
    fn default_tokens() -> Vec<Token> {
        [
            vec![Token::Map { len: None }],
            GDPRConsent::default_tokens(),
            vec![
                Token::Str("time"),
                Token::Str("1970-01-01T00:00:00Z"),
                Token::Str("from"),
                Token::Str(""),
                Token::MapEnd,
            ],
        ]
        .concat()
    }
}

impl DefaultTokens for AuthRequest {
    fn default_tokens() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "AuthRequest",
                len: 4,
            },
            Token::Str("type"),
            Token::Str("Auth"),
            Token::Str("type"),
            Token::Str("Login"),
            Token::Str("email"),
            Token::Str(""),
            Token::Str("password"),
            Token::Str(""),
            Token::StructEnd,
        ]
    }
}
