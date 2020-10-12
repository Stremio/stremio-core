use crate::types::addon::Descriptor;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;
use url::{form_urlencoded, Url};

pub type ExtraProp = (String, String);

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct ResourceRef {
    pub resource: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub extra: Vec<ExtraProp>,
}

impl ResourceRef {
    pub fn without_extra(resource: &str, type_: &str, id: &str) -> Self {
        ResourceRef {
            resource: resource.to_owned(),
            type_: type_.to_owned(),
            id: id.to_owned(),
            extra: vec![],
        }
    }
    pub fn with_extra(resource: &str, type_: &str, id: &str, extra: &[ExtraProp]) -> Self {
        ResourceRef {
            resource: resource.to_owned(),
            type_: type_.to_owned(),
            id: id.to_owned(),
            extra: extra.to_owned(),
        }
    }
    pub fn get_extra_first_value(&self, key: &str) -> Option<&String> {
        self.extra.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
    pub fn set_extra_unique(&mut self, key: &str, val: String) {
        let entry = self.extra.iter_mut().find(|(k, _)| k == key);
        match entry {
            Some(entry) => entry.1 = val,
            None => self.extra.push((key.to_owned(), val)),
        }
    }
    pub fn eq_no_extra(&self, other: &ResourceRef) -> bool {
        self.resource == other.resource && self.type_ == other.type_ && self.id == other.id
    }
}

impl fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "/{}/{}/{}",
            &utf8_percent_encode(&self.resource, NON_ALPHANUMERIC),
            &utf8_percent_encode(&self.type_, NON_ALPHANUMERIC),
            &utf8_percent_encode(&self.id, NON_ALPHANUMERIC)
        )?;
        if !self.extra.is_empty() {
            let mut extra_encoded = form_urlencoded::Serializer::new(String::new());
            for (k, v) in self.extra.iter() {
                extra_encoded.append_pair(&k, &v);
            }
            write!(f, "/{}", &extra_encoded.finish())?;
        }
        write!(f, ".json")
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct ResourceRequest {
    pub base: Url,
    pub path: ResourceRef,
}

impl ResourceRequest {
    pub fn new(base: Url, path: ResourceRef) -> Self {
        ResourceRequest { base, path }
    }
    pub fn eq_no_extra(&self, other: &ResourceRequest) -> bool {
        self.base == other.base && self.path.eq_no_extra(&other.path)
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum AggrRequest<'a> {
    AllCatalogs { extra: &'a Vec<ExtraProp> },
    AllOfResource(ResourceRef),
}

impl AggrRequest<'_> {
    pub fn plan<'a>(&self, addons: &'a [Descriptor]) -> Vec<(&'a Descriptor, ResourceRequest)> {
        match &self {
            AggrRequest::AllCatalogs { extra } => addons
                .iter()
                .map(|addon| {
                    addon
                        .manifest
                        .catalogs
                        .iter()
                        .filter(|cat| cat.is_extra_supported(&extra))
                        .map(move |cat| {
                            (
                                addon,
                                ResourceRequest::new(
                                    addon.transport_url.to_owned(),
                                    ResourceRef::with_extra("catalog", &cat.type_, &cat.id, extra),
                                ),
                            )
                        })
                })
                .flatten()
                .collect(),
            AggrRequest::AllOfResource(path) => addons
                .iter()
                .filter(|addon| addon.manifest.is_supported(&path))
                .map(|addon| {
                    (
                        addon,
                        ResourceRequest::new(addon.transport_url.to_owned(), path.to_owned()),
                    )
                })
                .collect(),
        }
    }
}
