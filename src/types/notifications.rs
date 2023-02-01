use std::collections::HashMap;

use chrono::{Utc, DateTime};

use crate::models::common::ResourceLoadable;

use super::{profile::UID, library::LibraryItem, resource::MetaItem};

/// Bucket for holding all the user's notifications locally.
/// 
/// It holds the loaded [`MetaItem`]s from each addon with indices
/// to know the priority of addon-loaded resources.
#[derive(Clone, Default, Debug)]
pub struct NotificationsBucket {
    pub uid: UID,
    ///
    /// [`HashMap`] Key is the [`LibraryItem`]`.id`.
    pub notifications: Vec<HashMap<String, Notification>>,

    /// each addon has it's own group
    /// These groups are ordered based on the addon indices which
    /// always ordered by installation order.
    ///
    /// `Cinemeta` is installed by default so it's always the first index in the user's addon list.
    pub groups: Vec<ResourceLoadable<Vec<MetaItem>>>,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: String,
    // last video released date
    pub latest_video: DateTime<Utc>,
    // last modified date
    pub modified: DateTime<Utc>,
}

// or MetaItem
// impl From<LibraryItem> for Notification {
// }