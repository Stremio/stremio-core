use crate::constants::URI_COMPONENT_ENCODE_SET;
use percent_encoding::utf8_percent_encode;
use std::borrow::Borrow;

pub fn query_params_encode<I, K, V>(query_params: I) -> String
where
    I: IntoIterator,
    I::Item: Borrow<(K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    query_params
        .into_iter()
        .map(|pair| {
            let (key, value) = pair.borrow();
            format!(
                "{}={}",
                utf8_percent_encode(key.as_ref(), URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(value.as_ref(), URI_COMPONENT_ENCODE_SET)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}
