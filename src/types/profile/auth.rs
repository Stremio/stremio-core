use crate::types::profile::User;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
pub struct AuthKey(pub String);

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
pub struct Auth {
    pub key: AuthKey,
    pub user: User,
}
