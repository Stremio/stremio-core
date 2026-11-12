use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Duration, Utc};
use semver::Version;
use sha2::{Digest, Sha256};

use crate::{
    constants::{
        REMOTE_CONFIG_DEFAULT_COOLDOWN_SECONDS, REMOTE_CONFIG_MAX_AGE,
        REMOTE_CONFIG_MAX_COOLDOWN_SECONDS,
    },
    types::{
        profile::{Profile, Settings},
        remote_config::{
            patch_settings, ActionDef, Assignment, AuthRule, ButtonDef, ClientInfo, CountriesRule,
            ExperimentAssignment, ExperimentUnit, LocaleCopy, NotificationDef, ParsedConfig,
            RemoteConfigBucket, RemoteNotification, RemoteNotificationButton, Targeting,
            VariantDef, VersionRange,
        },
    },
};

const ANONYMOUS_ACCOUNT_KEY: &str = "anonymous";
const DEFAULT_LOCALE: &str = "en-us";

pub fn point(value: &str) -> u64 {
    let digest = Sha256::digest(value.as_bytes());
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(bytes) % 10000
}

pub fn account_hash(salt: &str, account_id: &str) -> String {
    truncated_hash(&format!("{salt}:{account_id}"), 32)
}

pub fn account_id(profile: &Profile) -> Option<String> {
    profile.auth.as_ref().map(|auth| auth.user.id.0.to_owned())
}

pub fn account_key(client: &ClientInfo, profile: &Profile) -> String {
    match account_id(profile) {
        Some(account_id) => truncated_hash(
            &format!(
                "rc-account:{}:{account_id}",
                client.installation_id.as_deref().unwrap_or_default()
            ),
            16,
        ),
        None => ANONYMOUS_ACCOUNT_KEY.to_owned(),
    }
}

pub fn pick_variant(variants: &[VariantDef], point: u64) -> Option<&VariantDef> {
    let total = variants
        .iter()
        .try_fold(0u64, |total, variant| total.checked_add(variant.weight))?;
    if total == 0 {
        return None;
    }
    let mut running = 0u64;
    for variant in variants {
        let start = running.checked_mul(10000)? / total;
        running = running.checked_add(variant.weight)?;
        let end = running.checked_mul(10000)? / total;
        if (start..end).contains(&point) {
            return Some(variant);
        }
    }
    None
}

pub fn is_within(start: DateTime<Utc>, window: Duration, now: DateTime<Utc>) -> bool {
    start <= now
        && start
            .checked_add_signed(window)
            .is_some_and(|end| now < end)
}

pub fn set_settings_patches(
    button: &ButtonDef,
) -> impl Iterator<Item = &serde_json::Map<String, serde_json::Value>> {
    button.actions.iter().filter_map(|action| match action {
        ActionDef::SetSettings { patch } => Some(patch),
        _ => None,
    })
}

pub fn fresh_config(bucket: &RemoteConfigBucket, now: DateTime<Utc>) -> Option<ParsedConfig> {
    match (&bucket.config, bucket.fetched_at) {
        (Some(config), Some(fetched_at)) if is_within(fetched_at, REMOTE_CONFIG_MAX_AGE, now) => {
            Some(ParsedConfig::from(config))
        }
        _ => None,
    }
}

pub fn evaluate(
    bucket: &mut RemoteConfigBucket,
    client: &ClientInfo,
    profile: &Profile,
    now: DateTime<Utc>,
) -> (
    Vec<RemoteNotification>,
    BTreeMap<String, ExperimentAssignment>,
) {
    let Some(config) = fresh_config(bucket, now) else {
        return Default::default();
    };
    let mut audience = Audience {
        client,
        profile,
        now,
        settings: serde_json::to_value(&profile.settings).ok(),
        country: bucket.geo.as_ref().and_then(|geo| geo.country.to_owned()),
        language: normalize_language(
            &profile.settings.interface_language,
            &config.client.language_aliases,
        ),
        experiments: BTreeMap::new(),
    };

    let mut experiments = BTreeMap::new();
    for experiment in config
        .experiments
        .experiments
        .iter()
        .filter(|experiment| experiment.is_valid())
    {
        if !audience.matches(&experiment.id, &experiment.targeting) {
            continue;
        }
        let unit = match experiment.unit {
            ExperimentUnit::Installation => client.installation_id.to_owned(),
            ExperimentUnit::Account => account_id(profile),
        };
        let Some(unit) = unit else {
            continue;
        };
        let unit_key = truncated_hash(&format!("rc-unit:{unit}"), 16);
        let assignment_key = format!("{}:{unit_key}", experiment.id);
        let sticky = bucket
            .assignments
            .get(&assignment_key)
            .filter(|assignment| assignment.salt == experiment.salt)
            .and_then(|assignment| {
                experiment
                    .variants
                    .iter()
                    .find(|variant| variant.id == assignment.variant)
            });
        let variant = match sticky {
            Some(variant) => variant,
            None => {
                let traffic_point = point(&format!("{}:traffic:{unit}", experiment.salt));
                if traffic_point >= threshold(experiment.traffic_percent) {
                    continue;
                }
                let variant_point = point(&format!("{}:variant:{unit}", experiment.salt));
                let Some(variant) = pick_variant(&experiment.variants, variant_point) else {
                    continue;
                };
                bucket.assignments.insert(
                    assignment_key,
                    Assignment {
                        salt: experiment.salt.to_owned(),
                        variant: variant.id.to_owned(),
                        seen_at: now,
                    },
                );
                variant
            }
        };
        experiments.insert(
            experiment.id.to_owned(),
            ExperimentAssignment {
                variant: variant.id.to_owned(),
                payload: variant.payload.to_owned(),
                salt: experiment.salt.to_owned(),
                unit_key,
            },
        );
    }
    audience.experiments = experiments;

    let cooldown = Duration::seconds(
        config
            .notifications
            .cooldown_seconds
            .unwrap_or(REMOTE_CONFIG_DEFAULT_COOLDOWN_SECONDS)
            .min(REMOTE_CONFIG_MAX_COOLDOWN_SECONDS) as i64,
    );
    let in_cooldown = bucket
        .last_interaction_at
        .is_some_and(|last_interaction_at| is_within(last_interaction_at, cooldown, now));
    let dismissed = bucket.dismissed.get(&account_key(client, profile));
    let mut items = config
        .notifications
        .items
        .iter()
        .filter(|item| item.is_valid() && (!in_cooldown || item.targeting.accounts.is_some()))
        .filter(|item| client.notification_templates.contains(&item.template))
        .filter(|item| now < item.ends_at)
        .filter(|item| audience.matches(&item.id, &item.targeting))
        .filter(|item| {
            dismissed.map_or(true, |dismissed| {
                !dismissed.contains_key(&format!("{}:{}", item.id, item.rev))
            })
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(&b.id)));
    let notifications = items
        .into_iter()
        .filter_map(|item| resolve_notification(item, &profile.settings, &audience.language))
        .collect();

    (notifications, audience.experiments)
}

struct Audience<'a> {
    client: &'a ClientInfo,
    profile: &'a Profile,
    now: DateTime<Utc>,
    settings: Option<serde_json::Value>,
    country: Option<String>,
    language: String,
    experiments: BTreeMap<String, ExperimentAssignment>,
}

impl Audience<'_> {
    fn matches(&self, item_id: &str, targeting: &Targeting) -> bool {
        let user = self.profile.auth.as_ref().map(|auth| &auth.user);
        targeting
            .platforms
            .as_ref()
            .map_or(true, |platforms| platforms.contains(&self.client.platform))
            && targeting.os.as_ref().map_or(true, |os_list| {
                self.client
                    .os
                    .as_ref()
                    .is_some_and(|os| os_list.iter().any(|entry| entry.eq_ignore_ascii_case(os)))
            })
            && targeting.countries.as_ref().map_or(true, |rule| {
                self.country.as_ref().is_some_and(|country| match rule {
                    CountriesRule::Include(list) => {
                        list.iter().any(|entry| entry.eq_ignore_ascii_case(country))
                    }
                    CountriesRule::Exclude(list) => {
                        !list.iter().any(|entry| entry.eq_ignore_ascii_case(country))
                    }
                })
            })
            && targeting.languages.as_ref().map_or(true, |languages| {
                languages
                    .iter()
                    .any(|entry| language_matches(entry, &self.language))
            })
            && targeting.app_version.as_ref().map_or(true, |range| {
                version_matches(range, self.client.app_version.as_deref())
            })
            && targeting.shell_version.as_ref().map_or(true, |range| {
                version_matches(range, self.client.shell_version.as_deref())
            })
            && targeting
                .auth
                .map_or(true, |auth| (auth == AuthRule::LoggedIn) == user.is_some())
            && targeting.supporter.map_or(true, |supporter| {
                user.is_some_and(|user| {
                    user.premium_expire
                        .is_some_and(|premium_expire| premium_expire > self.now)
                        == supporter
                })
            })
            && targeting.registered.as_ref().map_or(true, |range| {
                user.is_some_and(|user| {
                    range
                        .after
                        .map_or(true, |after| after <= user.date_registered)
                        && range
                            .before
                            .map_or(true, |before| user.date_registered < before)
                })
            })
            && targeting.marketing_consent.map_or(true, |_| {
                user.is_some_and(|user| user.gdpr_consent.marketing)
            })
            && targeting.rollout_percent.map_or(true, |percent| {
                self.client
                    .installation_id
                    .as_ref()
                    .is_some_and(|installation_id| {
                        point(&format!("{item_id}:rollout:{installation_id}")) < threshold(percent)
                    })
            })
            && targeting.experiment.as_ref().map_or(true, |rule| {
                self.experiments
                    .get(&rule.id)
                    .is_some_and(|assignment| rule.variants.contains(&assignment.variant))
            })
            && targeting.accounts.as_ref().map_or(true, |rule| {
                account_id(self.profile).is_some_and(|account_id| {
                    let hash = account_hash(&rule.salt, &account_id);
                    rule.hashes
                        .iter()
                        .any(|entry| entry.eq_ignore_ascii_case(&hash))
                })
            })
            && targeting.settings.as_ref().map_or(true, |entries| {
                self.settings.as_ref().is_some_and(|settings| {
                    entries.iter().all(|(path, value)| {
                        path.split('.')
                            .try_fold(settings, |node, key| node.get(key))
                            == Some(value)
                    })
                })
            })
    }
}

fn resolve_notification(
    item: &NotificationDef,
    settings: &Settings,
    language: &str,
) -> Option<RemoteNotification> {
    let actions_valid = item.buttons.iter().all(|button| {
        let urls_valid = button.actions.iter().all(|action| match action {
            ActionDef::OpenUrl { url } => url.scheme() == "https",
            _ => true,
        });
        let mut patches = set_settings_patches(button).peekable();
        urls_valid && (patches.peek().is_none() || patch_settings(settings, patches).is_some())
    });
    if !actions_valid {
        return None;
    }
    let copy = resolve_copy(&item.content.locales, language, &item.buttons)?;
    Some(RemoteNotification {
        id: item.id.to_owned(),
        rev: item.rev,
        template: item.template.to_owned(),
        title: copy.title.to_owned()?,
        body: copy.body.to_owned(),
        image: item
            .content
            .image
            .to_owned()
            .filter(|image| image.scheme() == "https"),
        buttons: item
            .buttons
            .iter()
            .map(|button| RemoteNotificationButton {
                id: button.id.to_owned(),
                label: copy.buttons.get(&button.id).cloned().unwrap_or_default(),
                style: button.style.to_owned(),
                open_url: button.actions.iter().find_map(|action| match action {
                    ActionDef::OpenUrl { url } => Some(url.to_owned()),
                    _ => None,
                }),
                navigate: button.actions.iter().find_map(|action| match action {
                    ActionDef::Navigate { to } => Some(to.to_owned()),
                    _ => None,
                }),
            })
            .collect(),
    })
}

fn resolve_copy<'a>(
    locales: &'a HashMap<String, LocaleCopy>,
    language: &str,
    buttons: &[ButtonDef],
) -> Option<&'a LocaleCopy> {
    let is_complete = |copy: &&LocaleCopy| {
        copy.title.as_ref().is_some_and(|title| !title.is_empty())
            && buttons.iter().all(|button| {
                copy.buttons
                    .get(&button.id)
                    .is_some_and(|label| !label.is_empty())
            })
    };
    let find_tag = |tag: &str| {
        locales
            .iter()
            .find(|(locale, _)| locale.to_lowercase() == tag)
            .map(|(_, copy)| copy)
    };
    let primary = primary_subtag(language);
    find_tag(language)
        .or_else(|| {
            locales
                .iter()
                .filter(|(locale, _)| primary_subtag(&locale.to_lowercase()) == primary)
                .min_by(|(a, _), (b, _)| a.cmp(b))
                .map(|(_, copy)| copy)
        })
        .filter(is_complete)
        .or_else(|| find_tag(DEFAULT_LOCALE).filter(is_complete))
}

fn normalize_language(language: &str, aliases: &HashMap<String, String>) -> String {
    let language = aliases
        .get(language)
        .map(String::as_str)
        .unwrap_or(language);
    let language = if language == "eng" { "en-US" } else { language };
    language.to_lowercase()
}

fn language_matches(entry: &str, language: &str) -> bool {
    let entry = entry.to_lowercase();
    if entry.contains('-') {
        entry == language
    } else {
        entry == primary_subtag(language)
    }
}

fn primary_subtag(language: &str) -> &str {
    language.split('-').next().unwrap_or(language)
}

fn version_matches(range: &VersionRange, version: Option<&str>) -> bool {
    version
        .and_then(|version| Version::parse(version).ok())
        .is_some_and(|version| {
            range.min.as_ref().map_or(true, |min| &version >= min)
                && range.max.as_ref().map_or(true, |max| &version <= max)
        })
}

fn threshold(percent: f64) -> u64 {
    (percent * 100.0).round().clamp(0.0, 10000.0) as u64
}

fn truncated_hash(value: &str, length: usize) -> String {
    let mut hash = hex::encode(Sha256::digest(value.as_bytes()));
    hash.truncate(length);
    hash
}
