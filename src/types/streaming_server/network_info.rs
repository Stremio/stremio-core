use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    pub available_interfaces: Vec<String>,
}
