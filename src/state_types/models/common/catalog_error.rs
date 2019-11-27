use serde_derive::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum CatalogError {
    EmptyContent,
    UnexpectedResp,
    Other(String),
}
