use crate::types::profile::User;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct AuthKey(pub String);

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct Auth {
    pub key: AuthKey,
    pub user: User,
}
