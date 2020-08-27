use crate::types::profile::User;
use serde::{Deserialize, Serialize};

pub type AuthKey = String;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Auth {
    pub key: AuthKey,
    pub user: User,
}
