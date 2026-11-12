use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError, MapSkipError, VecSkipError};

use crate::{
    constants::{
        REMOTE_CONFIG_ASSIGNMENT_MAX_ABSENCE, REMOTE_CONFIG_MAX_ACCOUNT_KEYS,
        REMOTE_CONFIG_OUTBOX_CAPACITY, REMOTE_CONFIG_OUTBOX_MAX_AGE, REMOTE_CONFIG_RECORD_MAX_AGE,
    },
    types::remote_config::{is_within, ParsedConfig, RemoteConfigResponse},
};

#[serde_as]
#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RemoteConfigBucket {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub config: Option<RemoteConfigResponse>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub fetched_at: Option<DateTime<Utc>>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub geo: Option<Geo>,
    #[serde_as(deserialize_as = "DefaultOnError<MapSkipError<_, MapSkipError<_, _>>>")]
    pub dismissed: HashMap<String, HashMap<String, DateTime<Utc>>>,
    #[serde_as(deserialize_as = "DefaultOnError<MapSkipError<_, _>>")]
    pub assignments: HashMap<String, Assignment>,
    #[serde_as(deserialize_as = "DefaultOnError<MapSkipError<_, _>>")]
    pub sent: HashMap<String, DateTime<Utc>>,
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_>>")]
    pub outbox: Vec<QueuedEvent>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub next_seq: u64,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub last_interaction_at: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Geo {
    pub country: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Assignment {
    pub salt: String,
    pub variant: String,
    pub seen_at: DateTime<Utc>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct QueuedEvent {
    pub seq: u64,
    pub at: DateTime<Utc>,
    #[serde(flatten)]
    pub event: ConfigEvent,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ConfigEventKind {
    Notification,
    Experiment,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ConfigEvent {
    pub kind: ConfigEventKind,
    pub id: String,
    pub variant: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    pub event: String,
}

impl RemoteConfigBucket {
    pub fn queue_event_once(
        &mut self,
        key: String,
        event: ConfigEvent,
        now: DateTime<Utc>,
    ) -> bool {
        if self.sent.contains_key(&key) {
            return false;
        }
        if self.next_seq == u64::MAX {
            self.outbox.clear();
            self.next_seq = 0;
        }
        self.sent.insert(key, now);
        self.outbox.push(QueuedEvent {
            seq: self.next_seq,
            at: now,
            event,
        });
        self.next_seq += 1;
        self.outbox
            .retain(|queued| is_within(queued.at, REMOTE_CONFIG_OUTBOX_MAX_AGE, now));
        let overflow = self
            .outbox
            .len()
            .saturating_sub(REMOTE_CONFIG_OUTBOX_CAPACITY);
        self.outbox.drain(..overflow);
        true
    }

    pub fn union(&mut self, stored: RemoteConfigBucket) {
        for (account_key, stored_items) in stored.dismissed {
            let items = self.dismissed.entry(account_key).or_default();
            for (key, at) in stored_items {
                items.entry(key).or_insert(at);
            }
        }
        for (key, at) in stored.sent {
            self.sent.entry(key).or_insert(at);
        }
    }

    pub fn prune(&mut self, now: DateTime<Utc>) {
        let config = self.config.as_ref().map(ParsedConfig::from);
        let notification_ids = config
            .iter()
            .flat_map(|config| config.notifications.items.iter())
            .map(|item| item.id.as_str())
            .collect::<HashSet<_>>();
        let experiment_ids = config
            .iter()
            .flat_map(|config| config.experiments.experiments.iter())
            .map(|experiment| experiment.id.as_str())
            .collect::<HashSet<_>>();
        let is_recent = |at: &DateTime<Utc>| is_within(*at, REMOTE_CONFIG_RECORD_MAX_AGE, now);

        for items in self.dismissed.values_mut() {
            items.retain(|key, at| is_recent(at) || notification_ids.contains(item_id(key)));
        }
        self.dismissed.retain(|_, items| !items.is_empty());
        if self.dismissed.len() > REMOTE_CONFIG_MAX_ACCOUNT_KEYS {
            let mut account_keys = self
                .dismissed
                .iter()
                .map(|(account_key, items)| (items.values().max().copied(), account_key.to_owned()))
                .collect::<Vec<_>>();
            account_keys.sort_by(|a, b| b.cmp(a));
            for (_, account_key) in account_keys
                .into_iter()
                .skip(REMOTE_CONFIG_MAX_ACCOUNT_KEYS)
            {
                self.dismissed.remove(&account_key);
            }
        }

        self.sent.retain(|key, at| {
            let mut parts = key.split(':');
            is_recent(at)
                || match (parts.next(), parts.next()) {
                    (Some("n"), Some(id)) => notification_ids.contains(id),
                    (Some("e"), Some(id)) => experiment_ids.contains(id),
                    _ => false,
                }
        });

        for (key, assignment) in self.assignments.iter_mut() {
            if experiment_ids.contains(item_id(key)) {
                assignment.seen_at = now;
            }
        }
        self.assignments.retain(|_, assignment| {
            is_within(
                assignment.seen_at,
                REMOTE_CONFIG_ASSIGNMENT_MAX_ABSENCE,
                now,
            )
        });

        self.outbox
            .retain(|queued| is_within(queued.at, REMOTE_CONFIG_OUTBOX_MAX_AGE, now));
    }
}

fn item_id(key: &str) -> &str {
    key.split(':').next().unwrap_or(key)
}
