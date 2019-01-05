use serde_derive::*;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamItem {
    // @TODO: new spec
    external_url: String,
}
