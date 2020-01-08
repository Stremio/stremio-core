use crate::types::api::{AuthKey, User};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Auth {
    pub key: AuthKey,
    pub user: User,
}
