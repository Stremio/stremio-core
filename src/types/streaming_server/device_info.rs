use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError};

#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub available_hardware_accelerations: Vec<String>,
}
