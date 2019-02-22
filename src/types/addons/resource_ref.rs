use serde_derive::*;
use std::fmt;
use std::str::FromStr;
use url::form_urlencoded;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
pub type Extra = Vec<(String, String)>;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    pub resource: String,
    pub type_name: String,
    pub id: String,
    pub extra: Extra,
}
// @TODO test going to string/from string
impl fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "/{}/{}/{}",
            &utf8_percent_encode(&self.resource, DEFAULT_ENCODE_SET),
            &utf8_percent_encode(&self.type_name, DEFAULT_ENCODE_SET),
            &utf8_percent_encode(&self.id, DEFAULT_ENCODE_SET)
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
pub enum ParseResourceErr {
    DoesNotStartWithSlash,
    InvalidLength(usize),
    DecodeExtraErr,
}
impl FromStr for ResourceRef {
    type Err = ParseResourceErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("/") {
            return Err(ParseResourceErr::DoesNotStartWithSlash);
        }
        let components: Vec<&str> = s.split(',').skip(1).collect();
        match components.len() {
            // @TODO extra, utf8 percent decode
            3 => Ok(ResourceRef {
                resource: components[0].to_owned(),
                type_name: components[1].to_owned(),
                id: components[2].to_owned(),
                extra: vec![],
            }),
            4 => Ok(ResourceRef {
                resource: components[0].to_owned(),
                type_name: components[1].to_owned(),
                id: components[2].to_owned(),
                extra: vec![],
            }),
            i => Err(ParseResourceErr::InvalidLength(i)),
        }
    }
}

