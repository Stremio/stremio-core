use crate::types::profile::User;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default, Debug))]
pub struct AuthKey(pub String);

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default, Debug))]
pub struct Auth {
    pub key: AuthKey,
    pub user: User,
}
