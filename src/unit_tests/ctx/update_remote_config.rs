use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use chrono::{DateTime, Duration, TimeZone, Utc};
use enclose::enclose;
use futures::{channel::oneshot, future, FutureExt};
use serde_json::{json, Value};
use stremio_derive::Model;

use crate::{
    constants::{PROFILE_STORAGE_KEY, REMOTE_CONFIG_STORAGE_KEY},
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx, ActionRemoteConfig, Event},
        Env, EnvError, EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent, TryEnvFuture,
    },
    types::{
        api::{APIResult, GeoResponse, SuccessResponse},
        profile::{Auth, AuthKey, GDPRConsent, Profile, Settings, User},
        remote_config::{
            account_hash, pick_variant, point, Assignment, ConfigEvent, ConfigEventKind, Geo,
            QueuedEvent, RemoteConfigBucket, RemoteConfigResponse, RemoteNotification,
            RemoteNotificationButton, VariantDef,
        },
        True,
    },
    unit_tests::{
        default_client_info, default_fetch_handler, FetchHandler, Request, TestEnv, CLIENT_INFO,
        EVENTS, FETCH_HANDLER, NOW, REQUESTS, STORAGE,
    },
};

const CONFIG_URL: &str = "https://api.strem.io/config";
const GEO_URL: &str = "https://api.strem.io/api/getGeo";
const EVENTS_URL: &str = "https://api.strem.io/api/configEvents";

static GATES: Mutex<Vec<(&str, oneshot::Sender<()>)>> = Mutex::new(Vec::new());

fn gate(name: &'static str) -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();
    GATES.lock().unwrap().push((name, sender));
    receiver
}

fn open_gate(name: &str) {
    let mut gates = GATES.lock().unwrap();
    if let Some(index) = gates.iter().position(|(gate, _)| *gate == name) {
        let _ = gates.remove(index).1.send(());
    }
}

fn respond_after(
    name: &'static str,
    then_open: Option<&'static str>,
    response: Result<Box<dyn Any + Send>, EnvError>,
) -> TryEnvFuture<Box<dyn Any + Send>> {
    gate(name)
        .map(move |_| {
            if let Some(then_open) = then_open {
                open_gate(then_open);
            }
            response
        })
        .boxed_env()
}

fn respond_opening(
    name: &'static str,
    response: Result<Box<dyn Any + Send>, EnvError>,
) -> TryEnvFuture<Box<dyn Any + Send>> {
    future::lazy(move |_| {
        open_gate(name);
        response
    })
    .boxed_env()
}

#[derive(Model, Clone, Debug)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
}

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 20, 0, 0, 0).unwrap()
}

fn set_now(now: DateTime<Utc>) {
    *NOW.write().unwrap() = now;
}

fn handler(
    config: Option<RemoteConfigResponse>,
    country: Option<&str>,
    events_ok: bool,
) -> FetchHandler {
    let country = country.map(str::to_owned);
    Box::new(move |request: Request| match request.url.as_str() {
        CONFIG_URL => match &config {
            Some(config) => {
                future::ok(Box::new(config.to_owned()) as Box<dyn Any + Send>).boxed_env()
            }
            None => future::err(EnvError::Fetch("config unavailable".to_owned())).boxed_env(),
        },
        GEO_URL => match &country {
            Some(country) => future::ok(Box::new(APIResult::Ok(GeoResponse {
                country: Some(country.to_owned()),
            })) as Box<dyn Any + Send>)
            .boxed_env(),
            None => future::err(EnvError::Fetch("geo unavailable".to_owned())).boxed_env(),
        },
        EVENTS_URL if events_ok => {
            future::ok(
                Box::new(APIResult::Ok(SuccessResponse { success: True })) as Box<dyn Any + Send>
            )
            .boxed_env()
        }
        EVENTS_URL => future::err(EnvError::Fetch("events unavailable".to_owned())).boxed_env(),
        _ => default_fetch_handler(request),
    })
}

fn set_handler(config: Option<RemoteConfigResponse>, country: Option<&str>, events_ok: bool) {
    *FETCH_HANDLER.write().unwrap() = handler(config, country, events_ok);
}

fn notification(id: &str, targeting: Value) -> Value {
    json!({
        "v": 1,
        "id": id,
        "rev": 1,
        "template": "prompt",
        "endsAt": "2027-01-15T00:00:00Z",
        "targeting": targeting,
        "content": {
            "image": null,
            "locales": {
                "en-US": {
                    "title": "A new look",
                    "body": "Try it",
                    "buttons": { "try": "Try", "keep": "Keep" }
                },
                "bg-BG": {
                    "title": "Нов вид",
                    "buttons": { "try": "Опитай", "keep": "Остави" }
                }
            }
        },
        "buttons": [
            {
                "id": "try",
                "style": "primary",
                "actions": [{ "type": "setSettings", "patch": { "hideSpoilers": true } }]
            },
            { "id": "keep", "style": "secondary", "actions": [] }
        ]
    })
}

fn experiment(salt: &str, weights: [u64; 2], traffic_percent: f64) -> Value {
    json!({
        "v": 1,
        "id": "spotlight-copy",
        "active": true,
        "salt": salt,
        "unit": "installation",
        "trafficPercent": traffic_percent,
        "targeting": { "platforms": ["web", "desktop"] },
        "variants": [
            { "id": "control", "weight": weights[0] },
            { "id": "short", "weight": weights[1], "payload": { "bodyLines": 1 } }
        ],
        "events": ["clicked"]
    })
}

fn config_with_cooldown(
    items: Vec<Value>,
    experiments: Vec<Value>,
    cooldown_seconds: u64,
) -> RemoteConfigResponse {
    RemoteConfigResponse {
        client: json!({ "v": 1, "languageAliases": { "bul": "bg-BG" } }),
        notifications: json!({ "v": 1, "cooldownSeconds": cooldown_seconds, "items": items }),
        experiments: json!({ "experiments": experiments }),
    }
}

fn config(items: Vec<Value>, experiments: Vec<Value>) -> RemoteConfigResponse {
    config_with_cooldown(items, experiments, 0)
}

fn stored_bucket(config: RemoteConfigResponse) -> RemoteConfigBucket {
    RemoteConfigBucket {
        config: Some(config),
        fetched_at: Some(now()),
        ..Default::default()
    }
}

fn user(id: &str) -> User {
    User {
        id: id.into(),
        email: format!("{id}@example.com"),
        date_registered: Utc.with_ymd_and_hms(2021, 6, 1, 0, 0, 0).unwrap(),
        gdpr_consent: GDPRConsent {
            tos: true,
            privacy: true,
            marketing: true,
            from: None,
        },
        ..Default::default()
    }
}

fn logged_in(user: User) -> Profile {
    Profile {
        auth: Some(Auth {
            key: AuthKey("auth_key".to_owned()),
            user,
        }),
        ..Default::default()
    }
}

fn new_runtime(
    profile: Profile,
    bucket: Option<RemoteConfigBucket>,
) -> (
    Runtime<TestEnv, TestModel>,
    futures::channel::mpsc::Receiver<RuntimeEvent<TestEnv, TestModel>>,
) {
    let ctx = Ctx {
        profile,
        ..Default::default()
    }
    .with_remote_config::<TestEnv>(bucket);
    Runtime::<TestEnv, _>::new(TestModel { ctx }, vec![], 1000)
}

fn dispatch(runtime: &Runtime<TestEnv, TestModel>, action: ActionRemoteConfig) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::RemoteConfig(action)),
        })
    });
}

fn shown_ids(runtime: &Runtime<TestEnv, TestModel>) -> Vec<String> {
    runtime
        .model()
        .unwrap()
        .ctx
        .remote_config
        .notifications
        .iter()
        .map(|notification| notification.id.to_owned())
        .collect()
}

fn stored() -> Option<RemoteConfigBucket> {
    STORAGE
        .read()
        .unwrap()
        .get(REMOTE_CONFIG_STORAGE_KEY)
        .map(|data| serde_json::from_str(data).unwrap())
}

fn requests_to(url: &str) -> Vec<Request> {
    REQUESTS
        .read()
        .unwrap()
        .iter()
        .filter(|request| request.url == url)
        .cloned()
        .collect()
}

fn is_shown(targeting: Value, profile: Profile, country: Option<&str>) -> bool {
    let mut bucket = stored_bucket(config(
        vec![notification("ntf_spotl1ght", targeting)],
        vec![],
    ));
    bucket.geo = country.map(|country| Geo {
        country: Some(country.to_owned()),
        fetched_at: now(),
    });
    let (runtime, _rx) = new_runtime(profile, Some(bucket));
    !shown_ids(&runtime).is_empty()
}

#[test]
fn pull_exposes_matching_prompt_and_throttles_fetches() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let served = config(
        vec![notification(
            "ntf_spotl1ght",
            json!({ "platforms": ["web", "desktop"], "auth": "loggedIn" }),
        )],
        vec![],
    );
    set_handler(Some(served.to_owned()), Some("BG"), true);
    let (runtime, _rx) = new_runtime(logged_in(user("user_a")), None);
    assert!(
        shown_ids(&runtime).is_empty(),
        "Nothing before the first fetch"
    );

    dispatch(&runtime, ActionRemoteConfig::Pull);

    assert_eq!(
        runtime.model().unwrap().ctx.remote_config.notifications,
        vec![RemoteNotification {
            id: "ntf_spotl1ght".to_owned(),
            rev: 1,
            template: "prompt".to_owned(),
            title: "A new look".to_owned(),
            body: Some("Try it".to_owned()),
            image: None,
            buttons: vec![
                RemoteNotificationButton {
                    id: "try".to_owned(),
                    label: "Try".to_owned(),
                    style: "primary".to_owned(),
                    open_url: None,
                    navigate: None,
                },
                RemoteNotificationButton {
                    id: "keep".to_owned(),
                    label: "Keep".to_owned(),
                    style: "secondary".to_owned(),
                    open_url: None,
                    navigate: None,
                },
            ],
        }],
        "The prompt is exposed resolved to the interface language"
    );
    assert_eq!(requests_to(CONFIG_URL).len(), 1);
    assert_eq!(requests_to(CONFIG_URL)[0].method, "GET");
    assert_eq!(requests_to(GEO_URL).len(), 1);
    assert_eq!(requests_to(GEO_URL)[0].body, r#"{"type":"GetGeo"}"#);
    assert_eq!(
        stored().and_then(|bucket| bucket.config),
        Some(served),
        "The fetched config is stored under remote_config"
    );

    set_now(now() + Duration::seconds(10));
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        requests_to(CONFIG_URL).len(),
        1,
        "No request inside the refresh interval"
    );

    set_now(now() + Duration::seconds(301));
    set_handler(None, Some("BG"), true);
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        requests_to(CONFIG_URL).len(),
        2,
        "Refreshed after the interval"
    );
    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_spotl1ght"],
        "A failed fetch keeps the stored config"
    );

    set_now(now() + Duration::seconds(331));
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        requests_to(CONFIG_URL).len(),
        2,
        "A failed fetch is not retried for 60 seconds"
    );
}

#[test]
fn invalid_items_are_skipped_one_by_one() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let unknown_targeting_key = notification("ntf_targeting", json!({ "deviceRam": 4 }));
    let mut unknown_action = notification("ntf_action", json!({}));
    unknown_action["buttons"][1]["actions"] = json!([{ "type": "runScript", "code": "x" }]);
    let mut unknown_field = notification("ntf_field", json!({}));
    unknown_field["startsAt"] = json!("2026-11-01T00:00:00Z");
    let mut newer_version = notification("ntf_version", json!({}));
    newer_version["v"] = json!(2);
    let mut extra_action_field = notification("ntf_action_field", json!({}));
    extra_action_field["buttons"][0]["actions"][0]["silent"] = json!(true);
    let mut two_host_actions = notification("ntf_host_actions", json!({}));
    two_host_actions["buttons"][1]["actions"] = json!([
        { "type": "openUrl", "url": "https://www.stremio.com/" },
        { "type": "navigate", "to": "stremio:///settings" }
    ]);
    let mut host_action_with_patch = notification("ntf_link", json!({}));
    host_action_with_patch["buttons"][0]["actions"] = json!([
        { "type": "setSettings", "patch": { "hideSpoilers": true } },
        { "type": "openUrl", "url": "https://www.stremio.com/" }
    ]);
    let mut cosmetic_content = notification("ntf_valid", json!({}));
    cosmetic_content["content"]["theme"] = json!("dark");
    let website_experiment = json!({
        "id": "donate-pricing",
        "active": true,
        "variants": [
            { "id": "control", "weight": 50, "label": "Annual default" },
            { "id": "monthly-default", "weight": 50, "label": "Monthly default" }
        ]
    });
    let bucket = stored_bucket(config(
        vec![
            unknown_targeting_key,
            unknown_action,
            unknown_field,
            newer_version,
            extra_action_field,
            two_host_actions,
            host_action_with_patch,
            cosmetic_content,
        ],
        vec![
            website_experiment,
            experiment("spotlight-copy:1", [50, 50], 100.0),
        ],
    ));

    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));

    assert_eq!(shown_ids(&runtime), vec!["ntf_link", "ntf_valid"]);
    assert_eq!(
        runtime.model().unwrap().ctx.remote_config.notifications[0].buttons[0]
            .open_url
            .as_ref()
            .map(|url| url.as_str()),
        Some("https://www.stremio.com/"),
        "One host action may accompany a settings patch"
    );
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .remote_config
            .experiments
            .keys()
            .collect::<Vec<_>>(),
        vec!["spotlight-copy"]
    );
}

#[test]
fn targeting_rules() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let user_a = logged_in(user("user_a"));
    let supporter = logged_in(User {
        premium_expire: Some(now() + Duration::days(30)),
        ..user("user_a")
    });
    let expired_supporter = logged_in(User {
        premium_expire: Some(now() - Duration::days(1)),
        ..user("user_a")
    });
    let with_language = |language: &str| Profile {
        settings: Settings {
            interface_language: language.to_owned(),
            ..Settings::default()
        },
        ..Default::default()
    };
    let user_a_hash = account_hash("s3cr3tsalt", "user_a");

    let cases = vec![
        (
            json!({ "platforms": ["web"] }),
            Profile::default(),
            None,
            true,
        ),
        (
            json!({ "platforms": ["desktop"] }),
            Profile::default(),
            None,
            false,
        ),
        (json!({ "platforms": [] }), Profile::default(), None, false),
        (json!({ "os": ["windows"] }), Profile::default(), None, true),
        (json!({ "os": ["macos"] }), Profile::default(), None, false),
        (
            json!({ "auth": "loggedIn" }),
            Profile::default(),
            None,
            false,
        ),
        (json!({ "auth": "loggedIn" }), user_a.to_owned(), None, true),
        (
            json!({ "auth": "loggedOut" }),
            Profile::default(),
            None,
            true,
        ),
        (
            json!({ "supporter": true }),
            supporter.to_owned(),
            None,
            true,
        ),
        (json!({ "supporter": true }), expired_supporter, None, false),
        (
            json!({ "supporter": false }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "registered": { "after": "2021-01-01T00:00:00Z" } }),
            user_a.to_owned(),
            None,
            true,
        ),
        (
            json!({ "registered": { "before": "2021-01-01T00:00:00Z" } }),
            user_a.to_owned(),
            None,
            false,
        ),
        (
            json!({ "marketingConsent": true }),
            user_a.to_owned(),
            None,
            true,
        ),
        (
            json!({ "marketingConsent": true }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "languages": ["en"] }),
            with_language("eng"),
            None,
            true,
        ),
        (
            json!({ "languages": ["en-US"] }),
            with_language("eng"),
            None,
            true,
        ),
        (
            json!({ "languages": ["en-GB"] }),
            with_language("eng"),
            None,
            false,
        ),
        (
            json!({ "languages": ["bg"] }),
            with_language("bul"),
            None,
            true,
        ),
        (
            json!({ "languages": ["bg"] }),
            with_language("eng"),
            None,
            false,
        ),
        (
            json!({ "appVersion": { "min": "5.0.0" } }),
            Profile::default(),
            None,
            true,
        ),
        (
            json!({ "appVersion": { "min": "5.0.1" } }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "appVersion": { "min": "4.0.0", "max": "5.0.0" } }),
            Profile::default(),
            None,
            true,
        ),
        (
            json!({ "appVersion": { "max": "5.0.0-beta.45" } }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "shellVersion": { "min": "0.0.0" } }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "settings": { "hideSpoilers": false } }),
            Profile::default(),
            None,
            true,
        ),
        (
            json!({ "settings": { "hideSpoilers": true } }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "settings": { "boardSpotlight.enabled": false } }),
            Profile::default(),
            None,
            true,
        ),
        (
            json!({ "settings": { "unknownSetting.enabled": false } }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "accounts": { "salt": "s3cr3tsalt", "hashes": [user_a_hash] } }),
            user_a.to_owned(),
            None,
            true,
        ),
        (
            json!({ "accounts": { "salt": "s3cr3tsalt", "hashes": [user_a_hash] } }),
            logged_in(user("user_b")),
            None,
            false,
        ),
        (
            json!({ "countries": { "include": ["BG"] } }),
            Profile::default(),
            Some("BG"),
            true,
        ),
        (
            json!({ "countries": { "include": ["BG"] } }),
            Profile::default(),
            Some("US"),
            false,
        ),
        (
            json!({ "countries": { "exclude": ["US"] } }),
            Profile::default(),
            Some("BG"),
            true,
        ),
        (
            json!({ "countries": { "exclude": ["US"] } }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "countries": { "include": ["BG"] } }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "rolloutPercent": 88 }),
            Profile::default(),
            None,
            false,
        ),
        (
            json!({ "rolloutPercent": 88.09 }),
            Profile::default(),
            None,
            true,
        ),
        (
            json!({ "experiment": { "id": "spotlight-copy", "variants": ["short"] } }),
            Profile::default(),
            None,
            false,
        ),
    ];
    for (targeting, profile, country, expected) in cases {
        assert_eq!(
            is_shown(targeting.to_owned(), profile, country),
            expected,
            "targeting {targeting} with country {country:?}"
        );
    }
}

#[test]
fn country_rule_waits_for_geo() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let served = config(
        vec![notification(
            "ntf_spotl1ght",
            json!({ "countries": { "include": ["BG"] } }),
        )],
        vec![],
    );
    set_handler(Some(served.to_owned()), None, true);
    let (runtime, _rx) = new_runtime(Profile::default(), None);
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert!(
        shown_ids(&runtime).is_empty(),
        "Hidden while the country is unknown"
    );

    set_now(now() + Duration::seconds(301));
    set_handler(Some(served), Some("bg"), true);
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(shown_ids(&runtime), vec!["ntf_spotl1ght"]);
}

#[test]
fn assignment_points_and_ranges() {
    assert_eq!(point("ntf_spotl1ght:rollout:0123456789abcdef0123"), 8808);
    assert_eq!(point("spotlight-copy:1:traffic:0123456789abcdef0123"), 192);
    assert_eq!(point("spotlight-copy:1:variant:0123456789abcdef0123"), 5373);
    assert_eq!(point("ntf_spotl1ght:rollout:ffffffffffffffffffff"), 6349);
    assert_eq!(point("spotlight-copy:1:variant:ffffffffffffffffffff"), 1563);
    assert_eq!(
        account_hash("s3cr3tsalt", "5f1a2b3c4d5e6f7a8b9c0d1e"),
        "66731779429027839150c1cfaf49c18a"
    );

    let variants = |weights: &[u64]| {
        weights
            .iter()
            .enumerate()
            .map(|(index, weight)| VariantDef {
                id: index.to_string(),
                weight: *weight,
                label: None,
                payload: None,
            })
            .collect::<Vec<_>>()
    };
    let picked = |weights: &[u64], point: u64| {
        pick_variant(&variants(weights), point).map(|variant| variant.id.to_owned())
    };
    assert_eq!(picked(&[50, 50], 0).as_deref(), Some("0"));
    assert_eq!(picked(&[50, 50], 4999).as_deref(), Some("0"));
    assert_eq!(picked(&[50, 50], 5000).as_deref(), Some("1"));
    assert_eq!(picked(&[50, 50], 9999).as_deref(), Some("1"));
    assert_eq!(picked(&[1, 0, 3], 2499).as_deref(), Some("0"));
    assert_eq!(picked(&[1, 0, 3], 2500).as_deref(), Some("2"));
    assert_eq!(picked(&[1, 0, 3], 9999).as_deref(), Some("2"));
    assert_eq!(picked(&[0, 0], 0), None);
    assert_eq!(
        picked(&[u64::MAX, 1], 0),
        None,
        "Overflowing weights assign nobody"
    );
}

#[test]
fn experiment_assignment_is_sticky_per_salt() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let variant_of = |runtime: &Runtime<TestEnv, TestModel>| {
        runtime
            .model()
            .unwrap()
            .ctx
            .remote_config
            .experiments
            .get("spotlight-copy")
            .map(|assignment| assignment.variant.to_owned())
    };

    set_handler(
        Some(config(
            vec![],
            vec![experiment("spotlight-copy:1", [50, 50], 100.0)],
        )),
        Some("BG"),
        true,
    );
    let (runtime, _rx) = new_runtime(Profile::default(), None);
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        variant_of(&runtime).as_deref(),
        Some("short"),
        "Point 5373 of [50, 50]"
    );
    assert_eq!(
        runtime.model().unwrap().ctx.remote_config.experiments["spotlight-copy"].payload,
        Some(json!({ "bodyLines": 1 }))
    );

    set_now(now() + Duration::seconds(301));
    set_handler(
        Some(config(
            vec![],
            vec![experiment("spotlight-copy:1", [100, 0], 1.0)],
        )),
        Some("BG"),
        true,
    );
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        variant_of(&runtime).as_deref(),
        Some("short"),
        "Changing weights or lowering traffic keeps an assigned unit"
    );
    let (restarted, _rx) = new_runtime(Profile::default(), stored());
    assert_eq!(
        variant_of(&restarted).as_deref(),
        Some("short"),
        "The assignment survives a restart"
    );

    set_now(now() + Duration::seconds(602));
    set_handler(
        Some(config(
            vec![],
            vec![experiment("spotlight-copy:2", [100, 0], 100.0)],
        )),
        Some("BG"),
        true,
    );
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        variant_of(&runtime).as_deref(),
        Some("control"),
        "A new salt reassigns"
    );

    for (traffic_percent, expected) in [(0.0, false), (1.0, false), (2.0, true), (100.0, true)] {
        let bucket = stored_bucket(config(
            vec![],
            vec![experiment("spotlight-copy:1", [50, 50], traffic_percent)],
        ));
        let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));
        assert_eq!(
            variant_of(&runtime).is_some(),
            expected,
            "trafficPercent {traffic_percent} against traffic point 192"
        );
    }
}

#[test]
fn dismissal_is_per_account_and_revision() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    *FETCH_HANDLER.write().unwrap() = Box::new(|request: Request| match request.url.as_str() {
        "https://api.strem.io/api/logout" => {
            future::ok(
                Box::new(APIResult::Ok(SuccessResponse { success: True })) as Box<dyn Any + Send>
            )
            .boxed_env()
        }
        _ => handler(None, None, true)(request),
    });
    let served = config(vec![notification("ntf_spotl1ght", json!({}))], vec![]);
    let (runtime, _rx) = new_runtime(
        logged_in(user("user_a")),
        Some(stored_bucket(served.to_owned())),
    );
    assert_eq!(shown_ids(&runtime), vec!["ntf_spotl1ght"]);

    dispatch(
        &runtime,
        ActionRemoteConfig::DismissNotification("ntf_spotl1ght".to_owned()),
    );
    assert!(shown_ids(&runtime).is_empty());
    assert!(
        stored().is_some(),
        "The dismissal is written under remote_config"
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::Logout),
        })
    });
    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_spotl1ght"],
        "The logged-out person did not dismiss it"
    );

    let (same_account, _rx) = new_runtime(logged_in(user("user_a")), stored());
    assert!(
        shown_ids(&same_account).is_empty(),
        "The dismissal survives logout"
    );
    let (other_account, _rx) = new_runtime(logged_in(user("user_b")), stored());
    assert_eq!(
        shown_ids(&other_account),
        vec!["ntf_spotl1ght"],
        "Another account still sees it"
    );

    let mut bumped = notification("ntf_spotl1ght", json!({}));
    bumped["rev"] = json!(2);
    let bucket = RemoteConfigBucket {
        config: Some(config(vec![bumped], vec![])),
        ..stored().unwrap()
    };
    let (after_bump, _rx) = new_runtime(logged_in(user("user_a")), Some(bucket));
    assert_eq!(
        shown_ids(&after_bump),
        vec!["ntf_spotl1ght"],
        "A rev bump shows it again"
    );
}

#[test]
fn storage_push_keeps_dismissals_from_another_tab() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    set_handler(None, None, true);
    let served = config(
        vec![
            notification("ntf_first", json!({})),
            notification("ntf_second", json!({})),
        ],
        vec![],
    );
    let (this_tab, _rx) = new_runtime(Profile::default(), Some(stored_bucket(served.to_owned())));
    let (other_tab, _rx) = new_runtime(Profile::default(), Some(stored_bucket(served)));

    dispatch(
        &other_tab,
        ActionRemoteConfig::DismissNotification("ntf_second".to_owned()),
    );
    dispatch(
        &this_tab,
        ActionRemoteConfig::DismissNotification("ntf_first".to_owned()),
    );

    let (restarted, _rx) = new_runtime(Profile::default(), stored());
    assert!(
        shown_ids(&restarted).is_empty(),
        "Both dismissals are kept in storage"
    );
}

#[test]
fn notification_action_applies_settings_and_dismisses() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    set_handler(None, None, true);
    let bucket = stored_bucket(config(
        vec![notification(
            "ntf_spotl1ght",
            json!({ "settings": { "hideSpoilers": false } }),
        )],
        vec![],
    ));
    let (runtime, rx) = new_runtime(Profile::default(), Some(bucket));
    let runtime = Arc::new(RwLock::new(runtime));

    TestEnv::run_with_runtime(
        rx,
        runtime.clone(),
        enclose!((runtime) move || {
            runtime.read().unwrap().dispatch(RuntimeAction {
                field: None,
                action: Action::Ctx(ActionCtx::RemoteConfig(ActionRemoteConfig::NotificationAction {
                    id: "ntf_spotl1ght".to_owned(),
                    button: "try".to_owned(),
                })),
            })
        }),
    );

    let runtime = runtime.read().unwrap();
    let model = runtime.model().unwrap();
    assert!(
        model.ctx.profile.settings.hide_spoilers,
        "The setting is applied"
    );
    assert!(
        model.ctx.remote_config.notifications.is_empty(),
        "The prompt is dismissed"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .is_some_and(|data| serde_json::from_str::<Profile>(data)
                .unwrap()
                .settings
                .hide_spoilers),
        "The setting is persisted like any settings update"
    );
    assert!(
        EVENTS.read().unwrap().iter().any(|event| matches!(
            event.downcast_ref::<RuntimeEvent<TestEnv, TestModel>>(),
            Some(RuntimeEvent::CoreEvent(Event::SettingsUpdated { settings })) if settings.hide_spoilers
        )),
        "SettingsUpdated is emitted"
    );
    assert_eq!(
        requests_to(EVENTS_URL)
            .iter()
            .map(|request| serde_json::from_str::<Value>(&request.body).unwrap())
            .collect::<Vec<_>>(),
        vec![json!({
            "type": "ConfigEvents",
            "platform": "web",
            "events": [{ "kind": "notification", "id": "ntf_spotl1ght", "variant": "1", "event": "action:try" }]
        })]
    );
}

#[test]
fn unsafe_settings_patches_hide_the_prompt() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let patches = vec![
        (json!({ "hideSpoilers": true }), true),
        (json!({ "boardSpotlight": { "enabled": true } }), true),
        (json!({ "unknownSetting": { "enabled": true } }), false),
        (json!({ "playerType": "external" }), false),
        (
            json!({ "streamingServerUrl": "http://127.0.0.1:11470/" }),
            false,
        ),
        (json!({ "hideSpoilers": [true] }), false),
        (json!({ "hideSpoilers": "yes" }), false),
        (
            json!({ "subtitlesFont": "https://example.com/font" }),
            false,
        ),
        (json!({ "subtitlesFont": "x".repeat(65) }), false),
        (json!({ "interfaceScale": 300 }), false),
        (json!({ "sendCrashReports": false }), false),
        (
            json!({ "streamingServerWarningDismissed": "2026-11-20T00:00:00Z" }),
            false,
        ),
        (json!({}), false),
    ];
    for (patch, expected) in patches {
        let mut item = notification("ntf_spotl1ght", json!({}));
        item["buttons"][0]["actions"] = json!([{ "type": "setSettings", "patch": patch }]);
        let (runtime, _rx) = new_runtime(
            Profile::default(),
            Some(stored_bucket(config(vec![item], vec![]))),
        );
        assert_eq!(!shown_ids(&runtime).is_empty(), expected, "patch {patch}");
    }
}

#[test]
fn events_are_sent_once_and_retried_until_delivered() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    set_handler(None, None, false);
    let bucket = stored_bucket(config(
        vec![notification("ntf_spotl1ght", json!({}))],
        vec![],
    ));
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));
    let shown_body = json!({
        "type": "ConfigEvents",
        "platform": "web",
        "events": [{ "kind": "notification", "id": "ntf_spotl1ght", "variant": "1", "event": "shown" }]
    });
    let bodies = || {
        requests_to(EVENTS_URL)
            .iter()
            .map(|request| serde_json::from_str::<Value>(&request.body).unwrap())
            .collect::<Vec<_>>()
    };

    dispatch(
        &runtime,
        ActionRemoteConfig::NotificationShown("ntf_spotl1ght".to_owned()),
    );
    dispatch(
        &runtime,
        ActionRemoteConfig::NotificationShown("ntf_spotl1ght".to_owned()),
    );
    assert_eq!(
        bodies(),
        vec![shown_body.to_owned()],
        "Queued once, first delivery failed"
    );

    set_now(now() + Duration::seconds(10));
    set_handler(None, None, true);
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        bodies(),
        vec![shown_body.to_owned(), shown_body.to_owned()],
        "The next Pull retries"
    );

    set_now(now() + Duration::seconds(20));
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(bodies().len(), 2, "Delivered events leave the outbox");
    assert!(
        stored().is_some_and(|bucket| bucket.outbox.is_empty()),
        "The stored outbox is empty"
    );
}

#[test]
fn notification_gated_on_experiment_counts_its_exposure() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    set_handler(None, None, true);
    let bucket = stored_bucket(config(
        vec![notification(
            "ntf_spotl1ght",
            json!({ "experiment": { "id": "spotlight-copy", "variants": ["short"] } }),
        )],
        vec![experiment("spotlight-copy:1", [50, 50], 100.0)],
    ));
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));
    assert_eq!(shown_ids(&runtime), vec!["ntf_spotl1ght"]);

    dispatch(
        &runtime,
        ActionRemoteConfig::NotificationShown("ntf_spotl1ght".to_owned()),
    );
    dispatch(
        &runtime,
        ActionRemoteConfig::ExperimentExposed("spotlight-copy".to_owned()),
    );
    dispatch(
        &runtime,
        ActionRemoteConfig::ExperimentEvent {
            id: "spotlight-copy".to_owned(),
            event: "unknown".to_owned(),
        },
    );
    dispatch(
        &runtime,
        ActionRemoteConfig::ExperimentEvent {
            id: "spotlight-copy".to_owned(),
            event: "clicked".to_owned(),
        },
    );

    let events = requests_to(EVENTS_URL)
        .iter()
        .flat_map(|request| {
            serde_json::from_str::<Value>(&request.body).unwrap()["events"]
                .as_array()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        events,
        vec![
            json!({ "kind": "notification", "id": "ntf_spotl1ght", "variant": "1", "event": "shown" }),
            json!({ "kind": "experiment", "id": "spotlight-copy", "variant": "short", "run": "spotlight-copy:1", "event": "exposure" }),
            json!({ "kind": "experiment", "id": "spotlight-copy", "variant": "short", "run": "spotlight-copy:1", "event": "clicked" }),
        ]
    );
}

#[test]
fn exposure_requires_template_time_and_fresh_config() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let served = config(vec![notification("ntf_spotl1ght", json!({}))], vec![]);

    CLIENT_INFO.write().unwrap().notification_templates = vec![];
    let (runtime, _rx) = new_runtime(Profile::default(), Some(stored_bucket(served.to_owned())));
    assert!(
        shown_ids(&runtime).is_empty(),
        "An undeclared template exposes nothing"
    );
    *CLIENT_INFO.write().unwrap() = default_client_info();

    set_now(Utc.with_ymd_and_hms(2027, 1, 15, 0, 0, 0).unwrap());
    let bucket = RemoteConfigBucket {
        fetched_at: Some(Utc.with_ymd_and_hms(2027, 1, 14, 0, 0, 0).unwrap()),
        ..stored_bucket(served.to_owned())
    };
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));
    assert!(
        shown_ids(&runtime).is_empty(),
        "A passed endsAt hides the item"
    );

    set_now(now() + Duration::days(7));
    let bucket = stored_bucket(config(
        vec![notification("ntf_spotl1ght", json!({}))],
        vec![experiment("spotlight-copy:1", [50, 50], 100.0)],
    ));
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));
    let model = runtime.model().unwrap();
    assert!(
        model.ctx.remote_config.notifications.is_empty()
            && model.ctx.remote_config.experiments.is_empty(),
        "A config older than 7 days exposes nothing"
    );
}

#[test]
fn cooldown_holds_back_the_next_notification() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let mut first = notification("ntf_first", json!({}));
    first["priority"] = json!(1);
    let served = config_with_cooldown(
        vec![notification("ntf_second", json!({})), first],
        vec![],
        3600,
    );
    set_handler(Some(served.to_owned()), Some("BG"), true);
    let (runtime, _rx) = new_runtime(Profile::default(), Some(stored_bucket(served)));
    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_first", "ntf_second"],
        "Higher priority first"
    );

    dispatch(
        &runtime,
        ActionRemoteConfig::DismissNotification("ntf_first".to_owned()),
    );
    assert!(
        shown_ids(&runtime).is_empty(),
        "Withheld during the cooldown"
    );

    set_now(now() + Duration::seconds(3599));
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert!(shown_ids(&runtime).is_empty());

    set_now(now() + Duration::seconds(3600));
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(shown_ids(&runtime), vec!["ntf_second"]);
}

#[test]
fn garbage_bucket_fields_fall_back_to_defaults() {
    let bucket = serde_json::from_value::<RemoteConfigBucket>(json!({
        "config": 5,
        "fetched_at": "yesterday",
        "geo": [],
        "dismissed": 1,
        "assignments": "a",
        "sent": [],
        "outbox": {},
        "next_seq": "z",
        "last_interaction_at": true
    }))
    .expect("Bucket should deserialize");
    assert_eq!(bucket, RemoteConfigBucket::default());
}

#[test]
fn malformed_dismissals_are_skipped_one_by_one() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let mut bucket = serde_json::to_value(stored_bucket(config(
        vec![
            notification("ntf_first", json!({})),
            notification("ntf_second", json!({})),
        ],
        vec![],
    )))
    .unwrap();
    bucket["dismissed"] = json!({
        "anonymous": { "ntf_first:1": "2026-11-19T00:00:00Z", "ntf_second:1": "yesterday" },
        "broken": 5
    });
    let bucket = serde_json::from_value::<RemoteConfigBucket>(bucket).unwrap();
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));
    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_second"],
        "The valid dismissal survives its malformed neighbours"
    );
}

fn account_experiment(weights: [u64; 2]) -> Value {
    let mut experiment = experiment("spotlight-copy:1", weights, 100.0);
    experiment["unit"] = json!("account");
    experiment
}

fn sent_experiment_events() -> Vec<Value> {
    requests_to(EVENTS_URL)
        .iter()
        .flat_map(|request| {
            serde_json::from_str::<Value>(&request.body).unwrap()["events"]
                .as_array()
                .unwrap()
                .to_owned()
        })
        .collect()
}

#[test]
fn invalid_targeting_values_skip_the_item() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let no_marketing = logged_in(User {
        gdpr_consent: GDPRConsent {
            tos: true,
            privacy: true,
            marketing: false,
            from: None,
        },
        ..user("user_a")
    });
    let cases = vec![
        (json!({ "platforms": null }), Profile::default()),
        (json!({ "os": null }), Profile::default()),
        (json!({ "countries": null }), Profile::default()),
        (json!({ "languages": null }), Profile::default()),
        (json!({ "appVersion": null }), Profile::default()),
        (json!({ "appVersion": { "min": null } }), Profile::default()),
        (json!({ "shellVersion": null }), Profile::default()),
        (json!({ "auth": null }), Profile::default()),
        (json!({ "supporter": null }), Profile::default()),
        (json!({ "registered": null }), no_marketing.to_owned()),
        (
            json!({ "registered": { "after": null } }),
            no_marketing.to_owned(),
        ),
        (json!({ "marketingConsent": null }), no_marketing.to_owned()),
        (
            json!({ "marketingConsent": false }),
            no_marketing.to_owned(),
        ),
        (json!({ "rolloutPercent": null }), Profile::default()),
        (json!({ "experiment": null }), Profile::default()),
        (json!({ "accounts": null }), no_marketing.to_owned()),
        (json!({ "settings": null }), Profile::default()),
    ];
    for (targeting, profile) in cases {
        assert!(
            !is_shown(targeting.to_owned(), profile, Some("BG")),
            "targeting {targeting} is invalid"
        );
    }
    assert!(
        is_shown(
            json!({ "settings": { "playerType": null } }),
            Profile::default(),
            None
        ),
        "A null setting value is still a value to compare"
    );
}

#[test]
fn out_of_range_stored_values_do_not_panic() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let served = config(
        vec![notification("ntf_spotl1ght", json!({}))],
        vec![experiment("spotlight-copy:1", [50, 50], 100.0)],
    );
    set_handler(Some(served.to_owned()), Some("BG"), true);
    let far_future = DateTime::<Utc>::MAX_UTC - Duration::days(1);
    let bucket = RemoteConfigBucket {
        config: Some(served),
        fetched_at: Some(far_future),
        geo: Some(Geo {
            country: Some("BG".to_owned()),
            fetched_at: far_future,
        }),
        last_interaction_at: Some(far_future),
        next_seq: u64::MAX,
        ..Default::default()
    };

    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));
    assert!(
        shown_ids(&runtime).is_empty(),
        "A fetch time in the future counts as stale"
    );

    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(requests_to(CONFIG_URL).len(), 1, "A refresh is due");
    assert_eq!(requests_to(GEO_URL).len(), 1, "A future geo time is stale");
    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_spotl1ght"],
        "A future interaction time does not hold a cooldown"
    );

    dispatch(
        &runtime,
        ActionRemoteConfig::NotificationShown("ntf_spotl1ght".to_owned()),
    );
    assert_eq!(
        requests_to(EVENTS_URL).len(),
        1,
        "Queueing at the last sequence number still sends"
    );
}

#[test]
fn oversized_experiments_are_skipped() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let heavy = experiment("spotlight-copy:1", [10001, 1], 100.0);
    let mut crowded = experiment("spotlight-copy:1", [50, 50], 100.0);
    crowded["variants"] = json!((0..9)
        .map(|index| json!({ "id": format!("v{index}"), "weight": 1 }))
        .collect::<Vec<_>>());
    for experiment in [heavy, crowded] {
        let (runtime, _rx) = new_runtime(
            Profile::default(),
            Some(stored_bucket(config(vec![], vec![experiment]))),
        );
        assert!(runtime
            .model()
            .unwrap()
            .ctx
            .remote_config
            .experiments
            .is_empty());
    }
}

#[test]
fn account_unit_assignment_is_sticky_per_account() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    set_handler(None, None, true);
    let variant_of = |runtime: &Runtime<TestEnv, TestModel>| {
        runtime
            .model()
            .unwrap()
            .ctx
            .remote_config
            .experiments
            .get("spotlight-copy")
            .map(|assignment| assignment.variant.to_owned())
    };
    let expose = |runtime: &Runtime<TestEnv, TestModel>| {
        dispatch(
            runtime,
            ActionRemoteConfig::ExperimentExposed("spotlight-copy".to_owned()),
        )
    };

    let (logged_out, _rx) = new_runtime(
        Profile::default(),
        Some(stored_bucket(config(
            vec![],
            vec![account_experiment([50, 50])],
        ))),
    );
    assert_eq!(variant_of(&logged_out), None, "No account, no assignment");

    let (account_a, _rx) = new_runtime(
        logged_in(user("user_a")),
        Some(stored_bucket(config(
            vec![],
            vec![account_experiment([50, 50])],
        ))),
    );
    assert_eq!(
        variant_of(&account_a).as_deref(),
        Some("control"),
        "Point 566 of [50, 50]"
    );
    expose(&account_a);

    let reweighted = RemoteConfigBucket {
        config: Some(config(vec![], vec![account_experiment([0, 100])])),
        ..stored().unwrap()
    };
    let (account_b, _rx) = new_runtime(logged_in(user("user_b")), Some(reweighted));
    assert_eq!(variant_of(&account_b).as_deref(), Some("short"));
    expose(&account_b);

    let (account_a_again, _rx) = new_runtime(logged_in(user("user_a")), stored());
    assert_eq!(
        variant_of(&account_a_again).as_deref(),
        Some("control"),
        "Another account on the device does not replace this account's assignment"
    );
    expose(&account_a_again);

    assert_eq!(
        sent_experiment_events(),
        vec![
            json!({ "kind": "experiment", "id": "spotlight-copy", "variant": "control", "run": "spotlight-copy:1", "event": "exposure" }),
            json!({ "kind": "experiment", "id": "spotlight-copy", "variant": "short", "run": "spotlight-copy:1", "event": "exposure" }),
        ],
        "Each account reports its own exposure once"
    );
}

#[test]
fn testing_items_ignore_the_cooldown() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    set_handler(None, None, true);
    let testers = json!({ "accounts": { "salt": "s3cr3tsalt", "hashes": [account_hash("s3cr3tsalt", "user_a")] } });
    let bucket = stored_bucket(config_with_cooldown(
        vec![
            notification("ntf_live_a", json!({})),
            notification("ntf_live_b", json!({})),
            notification("ntf_test_a", testers.to_owned()),
            notification("ntf_test_b", testers),
        ],
        vec![],
        3600,
    ));
    let (runtime, _rx) = new_runtime(logged_in(user("user_a")), Some(bucket));

    dispatch(
        &runtime,
        ActionRemoteConfig::DismissNotification("ntf_test_a".to_owned()),
    );
    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_live_a", "ntf_live_b", "ntf_test_b"],
        "Dismissing a testing item starts no cooldown"
    );

    dispatch(
        &runtime,
        ActionRemoteConfig::DismissNotification("ntf_live_a".to_owned()),
    );
    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_test_b"],
        "Testing items stay visible during the cooldown"
    );
}

#[test]
fn stalled_requests_stop_blocking_after_a_minute() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let served = config(vec![notification("ntf_spotl1ght", json!({}))], vec![]);
    set_handler(Some(served.to_owned()), Some("BG"), true);
    let mut bucket = RemoteConfigBucket {
        geo: Some(Geo {
            country: Some("BG".to_owned()),
            fetched_at: now(),
        }),
        ..stored_bucket(served)
    };
    bucket.fetched_at = Some(now() - Duration::days(1));
    bucket.queue_event_once(
        "n:ntf_spotl1ght:1:shown".to_owned(),
        ConfigEvent {
            kind: ConfigEventKind::Notification,
            id: "ntf_spotl1ght".to_owned(),
            variant: "1".to_owned(),
            run: None,
            event: "shown".to_owned(),
        },
        now(),
    );
    let mut ctx = Ctx::default().with_remote_config::<TestEnv>(Some(bucket));
    ctx.remote_config.last_attempt_at = Some(now() - Duration::seconds(30));
    ctx.remote_config.flushing_since = Some(now() - Duration::seconds(30));
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(TestModel { ctx }, vec![], 1000);

    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert!(
        requests_to(CONFIG_URL).is_empty() && requests_to(EVENTS_URL).is_empty(),
        "Requests started 30 seconds ago are still in flight"
    );

    set_now(now() + Duration::seconds(30));
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        requests_to(CONFIG_URL).len(),
        1,
        "A stalled fetch is retried"
    );
    assert_eq!(
        requests_to(EVENTS_URL).len(),
        1,
        "A stalled flush is retried"
    );
}

#[test]
fn outbox_keeps_the_newest_fifty_events_of_the_last_week() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    set_handler(None, None, false);
    let event = |id: &str| ConfigEvent {
        kind: ConfigEventKind::Notification,
        id: id.to_owned(),
        variant: "1".to_owned(),
        run: None,
        event: "shown".to_owned(),
    };
    let mut bucket = stored_bucket(config(
        vec![notification("ntf_spotl1ght", json!({}))],
        vec![],
    ));
    bucket.outbox = (0..50)
        .map(|seq| QueuedEvent {
            seq,
            at: if seq == 1 {
                now() - Duration::days(8)
            } else {
                now() - Duration::days(6)
            },
            event: event(&format!("ntf_{seq}")),
        })
        .collect();
    bucket.next_seq = 50;
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));

    dispatch(
        &runtime,
        ActionRemoteConfig::NotificationShown("ntf_spotl1ght".to_owned()),
    );

    let outbox = stored().unwrap().outbox;
    assert_eq!(outbox.len(), 50, "At most 50 queued events");
    assert_eq!(
        outbox.iter().map(|queued| queued.seq).collect::<Vec<_>>(),
        (0..=50).filter(|seq| *seq != 1).collect::<Vec<_>>(),
        "Events older than 7 days are dropped first"
    );
    set_now(now() + Duration::seconds(10));
    set_handler(None, None, true);
    let (runtime, _rx) = new_runtime(Profile::default(), stored());
    dispatch(
        &runtime,
        ActionRemoteConfig::ExperimentExposed("missing".to_owned()),
    );
    dispatch(&runtime, ActionRemoteConfig::Pull);
    let batch_sizes = requests_to(EVENTS_URL)
        .iter()
        .skip(1)
        .map(|request| {
            serde_json::from_str::<Value>(&request.body).unwrap()["events"]
                .as_array()
                .unwrap()
                .len()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        batch_sizes,
        vec![20, 20, 10],
        "Events are sent in batches of 20 until the outbox is empty"
    );
}

#[test]
fn pruning_keeps_live_and_recent_records() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let served = config(
        vec![notification("ntf_live", json!({}))],
        vec![experiment("spotlight-copy:1", [50, 50], 100.0)],
    );
    set_handler(Some(served.to_owned()), Some("BG"), true);
    let days_ago = |days: i64| now() - Duration::days(days);
    let as_map = |entries: Vec<(&str, DateTime<Utc>)>| {
        entries
            .into_iter()
            .map(|(key, at)| (key.to_owned(), at))
            .collect::<HashMap<_, _>>()
    };
    let assignment = |seen_at| Assignment {
        salt: "spotlight-copy:1".to_owned(),
        variant: "control".to_owned(),
        seen_at,
    };

    let mut dismissed = (1..=8)
        .map(|index| {
            (
                format!("account_{index}"),
                as_map(vec![("ntf_live:1", days_ago(index))]),
            )
        })
        .collect::<HashMap<_, _>>();
    dismissed.insert(
        "account_0".to_owned(),
        as_map(vec![
            ("ntf_gone:1", days_ago(366)),
            ("ntf_gone:2", days_ago(10)),
            ("ntf_live:1", days_ago(400)),
            ("ntf_recent:1", now()),
        ]),
    );
    let bucket = RemoteConfigBucket {
        config: Some(served),
        fetched_at: Some(days_ago(1)),
        dismissed,
        sent: as_map(vec![
            ("n:ntf_gone:1:shown", days_ago(366)),
            ("n:ntf_live:1:shown", days_ago(400)),
            ("e:experiment_gone:unit:salt:exposure", days_ago(366)),
            (
                "e:spotlight-copy:unit:spotlight-copy:1:exposure",
                days_ago(400),
            ),
        ]),
        assignments: [
            (
                "spotlight-copy:unit_live".to_owned(),
                assignment(days_ago(40)),
            ),
            (
                "experiment_gone:unit_old".to_owned(),
                assignment(days_ago(31)),
            ),
            (
                "experiment_gone:unit_new".to_owned(),
                assignment(days_ago(29)),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    let other_tab = RemoteConfigBucket {
        dismissed: [(
            "account_0".to_owned(),
            as_map(vec![
                ("ntf_other_tab_old:1", days_ago(366)),
                ("ntf_other_tab:1", days_ago(1)),
            ]),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    STORAGE.write().unwrap().insert(
        REMOTE_CONFIG_STORAGE_KEY.to_owned(),
        serde_json::to_string(&other_tab).unwrap(),
    );
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));

    dispatch(&runtime, ActionRemoteConfig::Pull);

    let stored = stored().unwrap();
    let mut account_keys = stored.dismissed.keys().cloned().collect::<Vec<_>>();
    account_keys.sort();
    assert_eq!(
        account_keys,
        (0..=7)
            .map(|index| format!("account_{index}"))
            .collect::<Vec<_>>(),
        "Only the 8 most recently used account keys are kept"
    );
    let mut account_0 = stored.dismissed["account_0"]
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    account_0.sort();
    assert_eq!(
        account_0,
        vec![
            "ntf_gone:2",
            "ntf_live:1",
            "ntf_other_tab:1",
            "ntf_recent:1"
        ],
        "Old dismissals of removed items are pruned, also after the union"
    );
    let mut sent = stored.sent.keys().cloned().collect::<Vec<_>>();
    sent.sort();
    assert_eq!(
        sent,
        vec![
            "e:spotlight-copy:unit:spotlight-copy:1:exposure",
            "n:ntf_live:1:shown"
        ]
    );
    assert!(
        stored.assignments.contains_key("spotlight-copy:unit_live")
            && stored.assignments.contains_key("experiment_gone:unit_new")
            && !stored.assignments.contains_key("experiment_gone:unit_old"),
        "Assignments of experiments absent for 30 days are pruned"
    );
}

fn runtime_action(action: ActionRemoteConfig) -> RuntimeAction<TestEnv, TestModel> {
    RuntimeAction {
        field: None,
        action: Action::Ctx(ActionCtx::RemoteConfig(action)),
    }
}

fn complete_two_fetches_in_reverse(
    url: &'static str,
    older: Box<dyn Fn() -> Result<Box<dyn Any + Send>, EnvError> + Send + Sync>,
    newer: Box<dyn Fn() -> Result<Box<dyn Any + Send>, EnvError> + Send + Sync>,
    served: RemoteConfigResponse,
) -> Runtime<TestEnv, TestModel> {
    *FETCH_HANDLER.write().unwrap() =
        Box::new(move |request: Request| match request.url.as_str() {
            requested if requested == url && requests_to(url).len() == 1 => {
                respond_after("older", None, older())
            }
            requested if requested == url => respond_opening("older", newer()),
            CONFIG_URL => {
                future::ok(Box::new(served.to_owned()) as Box<dyn Any + Send>).boxed_env()
            }
            GEO_URL => future::ok(Box::new(APIResult::Ok(GeoResponse {
                country: Some("BG".to_owned()),
            })) as Box<dyn Any + Send>)
            .boxed_env(),
            _ => default_fetch_handler(request),
        });
    let (runtime, _rx) = new_runtime(Profile::default(), None);
    TestEnv::run(|| {
        runtime.dispatch(runtime_action(ActionRemoteConfig::Pull));
        set_now(now() + Duration::seconds(61));
        runtime.dispatch(runtime_action(ActionRemoteConfig::Pull));
    });
    runtime
}

fn config_response(config: RemoteConfigResponse) -> Result<Box<dyn Any + Send>, EnvError> {
    Ok(Box::new(config))
}

fn geo_response(country: &str) -> Result<Box<dyn Any + Send>, EnvError> {
    Ok(Box::new(APIResult::Ok(GeoResponse {
        country: Some(country.to_owned()),
    })))
}

fn unavailable() -> Result<Box<dyn Any + Send>, EnvError> {
    Err(EnvError::Fetch("unavailable".to_owned()))
}

#[test]
fn an_older_config_arriving_late_is_ignored() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let runtime = complete_two_fetches_in_reverse(
        CONFIG_URL,
        Box::new(|| config_response(config(vec![notification("ntf_paused", json!({}))], vec![]))),
        Box::new(|| config_response(config(vec![], vec![]))),
        config(vec![], vec![]),
    );

    assert_eq!(requests_to(CONFIG_URL).len(), 2);
    assert!(
        shown_ids(&runtime).is_empty(),
        "The older body that arrives last does not restore a removed item"
    );
    assert_eq!(
        stored().and_then(|bucket| bucket.config),
        Some(config(vec![], vec![])),
        "The newest config stays stored"
    );
}

#[test]
fn a_lone_older_config_success_is_applied() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let runtime = complete_two_fetches_in_reverse(
        CONFIG_URL,
        Box::new(|| config_response(config(vec![notification("ntf_live", json!({}))], vec![]))),
        Box::new(unavailable),
        config(vec![], vec![]),
    );

    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_live"],
        "A late success is kept when the newer attempt failed"
    );
}

#[test]
fn an_older_geo_answer_arriving_late_is_ignored() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let runtime = complete_two_fetches_in_reverse(
        GEO_URL,
        Box::new(|| geo_response("US")),
        Box::new(|| geo_response("BG")),
        config(
            vec![notification(
                "ntf_bg",
                json!({ "countries": { "include": ["BG"] } }),
            )],
            vec![],
        ),
    );

    assert_eq!(requests_to(GEO_URL).len(), 2);
    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_bg"],
        "The newer country is kept"
    );
}

#[test]
fn a_lone_older_geo_success_is_applied() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let runtime = complete_two_fetches_in_reverse(
        GEO_URL,
        Box::new(|| geo_response("BG")),
        Box::new(unavailable),
        config(
            vec![notification(
                "ntf_bg",
                json!({ "countries": { "include": ["BG"] } }),
            )],
            vec![],
        ),
    );

    assert_eq!(
        shown_ids(&runtime),
        vec!["ntf_bg"],
        "A late country is kept when the newer attempt failed"
    );
}

#[test]
fn a_late_flush_success_keeps_the_newer_flush_in_flight() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    let success = || -> Result<Box<dyn Any + Send>, EnvError> {
        Ok(Box::new(APIResult::Ok(SuccessResponse { success: True })))
    };
    *FETCH_HANDLER.write().unwrap() =
        Box::new(move |request: Request| match request.url.as_str() {
            EVENTS_URL => match requests_to(EVENTS_URL).len() {
                1 => respond_after("first", Some("first_done"), success()),
                2 => {
                    let second = gate("second");
                    future::lazy(|_| open_gate("first"))
                        .then(move |_| second.map(move |_| success()))
                        .boxed_env()
                }
                _ => future::ready(success()).boxed_env(),
            },
            _ => default_fetch_handler(request),
        });
    let bucket = stored_bucket(config(
        vec![
            notification("ntf_first", json!({})),
            notification("ntf_second", json!({})),
        ],
        vec![],
    ));
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));
    let runtime = Arc::new(RwLock::new(runtime));
    let requests_while_held = Arc::new(Mutex::new(0));

    TestEnv::run(enclose!((runtime, requests_while_held) move || {
        let first_done = gate("first_done");
        runtime.read().unwrap().dispatch(runtime_action(
            ActionRemoteConfig::NotificationShown("ntf_first".to_owned()),
        ));
        set_now(now() + Duration::seconds(61));
        runtime.read().unwrap().dispatch(runtime_action(ActionRemoteConfig::Pull));
        TestEnv::exec_concurrent(async move {
            let _ = first_done.await;
            runtime.read().unwrap().dispatch(runtime_action(
                ActionRemoteConfig::NotificationShown("ntf_second".to_owned()),
            ));
            *requests_while_held.lock().unwrap() = requests_to(EVENTS_URL).len();
            open_gate("second");
        });
    }));

    assert_eq!(
        *requests_while_held.lock().unwrap(),
        2,
        "A late success does not let a third flush start while the newer one is in flight"
    );
    assert_eq!(
        requests_to(EVENTS_URL).len(),
        3,
        "The event queued meanwhile goes out once the newer flush succeeds"
    );
}

#[test]
fn a_superseded_flush_drops_only_its_own_events() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    *FETCH_HANDLER.write().unwrap() = Box::new(|request: Request| match request.url.as_str() {
        EVENTS_URL if requests_to(EVENTS_URL).len() == 1 => respond_after(
            "older",
            None,
            Ok(Box::new(APIResult::Ok(SuccessResponse { success: True }))),
        ),
        EVENTS_URL => respond_opening(
            "older",
            Err(EnvError::Fetch("events unavailable".to_owned())),
        ),
        _ => default_fetch_handler(request),
    });
    let bucket = stored_bucket(config(
        vec![notification("ntf_spotl1ght", json!({}))],
        vec![],
    ));
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));

    TestEnv::run(|| {
        runtime.dispatch(runtime_action(ActionRemoteConfig::NotificationShown(
            "ntf_spotl1ght".to_owned(),
        )));
        runtime.dispatch(runtime_action(ActionRemoteConfig::DismissNotification(
            "ntf_spotl1ght".to_owned(),
        )));
        set_now(now() + Duration::seconds(61));
        runtime.dispatch(runtime_action(ActionRemoteConfig::Pull));
    });

    assert_eq!(
        stored()
            .unwrap()
            .outbox
            .iter()
            .map(|queued| queued.event.event.to_owned())
            .collect::<Vec<_>>(),
        vec!["dismissed"],
        "The late success removes only the event it delivered"
    );
    assert_eq!(
        serde_json::from_str::<Value>(&requests_to(EVENTS_URL)[2].body).unwrap()["events"],
        json!([{ "kind": "notification", "id": "ntf_spotl1ght", "variant": "1", "event": "dismissed" }]),
        "The late success sends what is left right away"
    );

    set_now(now() + Duration::seconds(70));
    set_handler(None, None, true);
    dispatch(&runtime, ActionRemoteConfig::Pull);
    assert_eq!(
        serde_json::from_str::<Value>(&requests_to(EVENTS_URL).last().unwrap().body).unwrap()
            ["events"],
        json!([{ "kind": "notification", "id": "ntf_spotl1ght", "variant": "1", "event": "dismissed" }]),
        "The remaining event is delivered on the next Pull"
    );
}

#[test]
fn malformed_records_are_skipped_one_by_one() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    set_handler(None, None, true);
    let mut bucket = serde_json::to_value(stored_bucket(config(
        vec![notification("ntf_spotl1ght", json!({}))],
        vec![experiment("spotlight-copy:1", [50, 50], 100.0)],
    )))
    .unwrap();
    bucket["assignments"] = json!({
        "spotlight-copy:9721add9c54b86ae": {
            "salt": "spotlight-copy:1",
            "variant": "control",
            "seen_at": "2026-11-19T00:00:00Z"
        },
        "broken": 5
    });
    bucket["sent"] = json!({
        "n:ntf_spotl1ght:1:shown": "2026-11-19T00:00:00Z",
        "broken": "yesterday"
    });
    bucket["outbox"] = json!([
        {
            "seq": 0,
            "at": "2026-11-19T00:00:00Z",
            "kind": "notification",
            "id": "ntf_queued",
            "variant": "1",
            "event": "shown"
        },
        { "seq": "broken" }
    ]);
    bucket["next_seq"] = json!(1);
    let bucket = serde_json::from_value::<RemoteConfigBucket>(bucket).unwrap();
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));

    dispatch(
        &runtime,
        ActionRemoteConfig::NotificationShown("ntf_spotl1ght".to_owned()),
    );
    dispatch(
        &runtime,
        ActionRemoteConfig::ExperimentExposed("spotlight-copy".to_owned()),
    );

    assert_eq!(
        sent_experiment_events(),
        vec![
            json!({ "kind": "notification", "id": "ntf_queued", "variant": "1", "event": "shown" }),
            json!({ "kind": "experiment", "id": "spotlight-copy", "variant": "control", "run": "spotlight-copy:1", "event": "exposure" }),
        ],
        "The valid outbox entry, sent key and sticky assignment survive their malformed neighbours"
    );
}

#[test]
fn events_queued_during_a_flush_are_sent_when_it_succeeds() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    set_now(now());
    *FETCH_HANDLER.write().unwrap() = Box::new(|request: Request| match request.url.as_str() {
        EVENTS_URL if requests_to(EVENTS_URL).len() == 1 => respond_after(
            "first",
            None,
            Ok(Box::new(APIResult::Ok(SuccessResponse { success: True }))),
        ),
        EVENTS_URL => future::ok(
            Box::new(APIResult::Ok(SuccessResponse { success: True })) as Box<dyn Any + Send>
        )
        .boxed_env(),
        _ => default_fetch_handler(request),
    });
    let bucket = stored_bucket(config(
        vec![
            notification("ntf_first", json!({})),
            notification("ntf_second", json!({})),
        ],
        vec![],
    ));
    let (runtime, _rx) = new_runtime(Profile::default(), Some(bucket));

    TestEnv::run(|| {
        runtime.dispatch(runtime_action(ActionRemoteConfig::NotificationShown(
            "ntf_first".to_owned(),
        )));
        runtime.dispatch(runtime_action(ActionRemoteConfig::NotificationShown(
            "ntf_second".to_owned(),
        )));
        open_gate("first");
    });

    assert_eq!(
        requests_to(EVENTS_URL)
            .iter()
            .map(
                |request| serde_json::from_str::<Value>(&request.body).unwrap()["events"][0]["id"]
                    .to_owned()
            )
            .collect::<Vec<_>>(),
        vec![json!("ntf_first"), json!("ntf_second")],
        "The second event goes out as soon as the first flush succeeds, without a Pull"
    );
    assert!(stored().is_some_and(|bucket| bucket.outbox.is_empty()));
}
