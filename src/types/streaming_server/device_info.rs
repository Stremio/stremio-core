use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub available_hardware_accelerations: Vec<String>,
}
