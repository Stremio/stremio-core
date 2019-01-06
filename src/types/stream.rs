use serde_derive::*;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    // @TODO fill in all the props
    // @TODO: new spec
    url: Option<String>,
    external_url: Option<String>,
    info_hash: Option<String>,
}
