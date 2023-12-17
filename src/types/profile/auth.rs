use crate::types::profile::User;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct AuthKey(pub String);

impl ToString for AuthKey {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct Auth {
    pub key: AuthKey,
    pub user: User,
}
