use serde_derive::*;
use std::fmt;
use std::str::FromStr;
use url::form_urlencoded;
use url::percent_encoding::{percent_decode, utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
pub type ExtraProp = (String, String);

// ResourceRef is the type that represents a reference to a specific resource path
// in the addon system
// It can be stringified and parsed from a string, which is used by the addon transports

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    pub resource: String,
    pub type_name: String,
    pub id: String,
    pub extra: Vec<ExtraProp>,
}

impl ResourceRef {
    pub fn without_extra(resource: &str, type_name: &str, id: &str) -> Self {
        ResourceRef {
            resource: resource.to_owned(),
            type_name: type_name.to_owned(),
            id: id.to_owned(),
            extra: vec![],
        }
    }
    pub fn with_extra(resource: &str, type_name: &str, id: &str, extra: &[ExtraProp]) -> Self {
        ResourceRef {
            resource: resource.to_owned(),
            type_name: type_name.to_owned(),
            id: id.to_owned(),
            extra: extra.to_owned(),
        }
    }
}

impl fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "/{}/{}/{}",
            &utf8_percent_encode(&self.resource, PATH_SEGMENT_ENCODE_SET),
            &utf8_percent_encode(&self.type_name, PATH_SEGMENT_ENCODE_SET),
            &utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET)
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

#[derive(Debug)]
pub enum ParseResourceErr {
    WrongPrefix,
    WrongSuffix,
    InvalidLength(usize),
    DecodeErr,
}
impl FromStr for ResourceRef {
    type Err = ParseResourceErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('/') {
            return Err(ParseResourceErr::WrongPrefix);
        }
        if !s.ends_with(".json") {
            return Err(ParseResourceErr::WrongSuffix);
        }
        let components: Vec<&str> = s.trim_end_matches(".json").split('/').skip(1).collect();
        match components.len() {
            3 | 4 => Ok(ResourceRef {
                resource: parse_component(components[0])?,
                type_name: parse_component(components[1])?,
                id: parse_component(components[2])?,
                extra: components
                    .get(3)
                    .map(|e| form_urlencoded::parse(e.as_bytes()).into_owned().collect())
                    .unwrap_or_else(|| vec![]),
            }),
            i => Err(ParseResourceErr::InvalidLength(i)),
        }
    }
}
fn parse_component(s: &str) -> Result<String, ParseResourceErr> {
    Ok(percent_decode(s.as_bytes())
        .decode_utf8()
        .map_err(|_| ParseResourceErr::DecodeErr)?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_extra() {
        // We're using UTF8, slashes and dots in the ID to test if we're properly URL path encoding
        let r = ResourceRef::without_extra("catalog", "movie", "top/лол/.f");
        assert_eq!(r, ResourceRef::from_str(&r.to_string()).unwrap());
    }

    #[test]
    fn with_extra() {
        let extra = &[
            ("search".into(), "тест".into()),
            ("another".into(), "/something/".into()),
        ];
        let r = ResourceRef::with_extra("catalog", "movie", "top/лол.f", extra);
        assert_eq!(r, ResourceRef::from_str(&r.to_string()).unwrap());
    }

    #[test]
    fn compatible_with_js() {
        let extra = &[
            ("search".into(), "the office".into()),
            ("foo".into(), "bar".into()),
        ];
        assert_eq!(
            "/catalog/series/top/search=the+office&foo=bar.json",
            ResourceRef::with_extra("catalog", "series", "top", extra).to_string()
        );
    }
}
