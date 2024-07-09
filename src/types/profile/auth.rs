use std::fmt::Display;

use crate::types::profile::User;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct AuthKey(pub String);

impl Display for AuthKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct Auth {
    pub key: AuthKey,
    pub user: User,
}
