use crate::types::addon::{Descriptor, ExtraProp};
use derive_more::{From, Into};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use std::fmt;
use url::{form_urlencoded, Url};

#[derive(Clone, From, Into, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(from = "(String, String)", into = "(String, String)")]
pub struct ExtraValue {
    pub name: String,
    pub value: String,
}

pub trait ExtraExt {
    fn extend_one(self, prop: &ExtraProp, value: Option<ExtraValue>) -> Self;
}

impl ExtraExt for Vec<ExtraValue> {
    fn extend_one(self, prop: &ExtraProp, value: Option<ExtraValue>) -> Self {
        if value.as_ref().map(|ev| &ev.name) != Some(&prop.name) {
            return self;
        }

        let (extra, other_extra) = self
            .into_iter()
            .partition::<Vec<ExtraValue>, _>(|ev| ev.name == prop.name);
        let extra = match value {
            Some(value) if *prop.options_limit == 1 => vec![value],
            Some(value) if *prop.options_limit > 1 => {
                if extra.iter().any(|ev| ev.value == value.value) {
                    extra
                        .into_iter()
                        .filter(|ev| ev.value != value.value)
                        .collect()
                } else {
                    extra.into_iter().chain(vec![value]).collect()
                }
            }
            None if !prop.is_required => vec![],
            _ => extra,
        };
        extra.into_iter().chain(other_extra).collect()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct ResourceRef {
    pub resource: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub extra: Vec<ExtraValue>,
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
    pub fn with_extra(resource: &str, type_: &str, id: &str, extra: &[ExtraValue]) -> Self {
        ResourceRef {
            resource: resource.to_owned(),
            type_: type_.to_owned(),
            id: id.to_owned(),
            extra: extra.to_owned(),
        }
    }
    pub fn get_extra_first_value(&self, name: &str) -> Option<&String> {
        self.extra
            .iter()
            .find(|extra_value| extra_value.name == name)
            .map(|extra_value| &extra_value.value)
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
            for ExtraValue { name, value } in self.extra.iter() {
                extra_encoded.append_pair(&name, &value);
            }
            write!(f, "/{}", &extra_encoded.finish())?;
        }
        write!(f, ".json")
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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
    AllCatalogs { extra: &'a Vec<ExtraValue> },
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
                        .filter(|catalog| catalog.is_extra_supported(&extra))
                        .map(move |catalog| {
                            (
                                addon,
                                ResourceRequest::new(
                                    addon.transport_url.to_owned(),
                                    ResourceRef::with_extra(
                                        "catalog",
                                        &catalog.type_,
                                        &catalog.id,
                                        extra,
                                    ),
                                ),
                            )
                        })
                })
                .flatten()
                .collect(),
            AggrRequest::AllOfResource(path) => addons
                .iter()
                .filter(|addon| addon.manifest.is_resource_supported(&path))
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
