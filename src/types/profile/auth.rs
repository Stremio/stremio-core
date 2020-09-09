use crate::types::profile::User;
use serde::{Deserialize, Serialize};

pub type AuthKey = String;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct Auth {
    pub key: AuthKey,
    pub user: User,
}
