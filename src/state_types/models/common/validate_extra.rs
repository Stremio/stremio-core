use crate::constants::{SEARCH_EXTRA_NAME, SKIP_EXTRA_NAME};
use crate::types::addons::ExtraProp;

pub fn validate_extra(extra: &[ExtraProp], page_size: &Option<usize>) -> Vec<ExtraProp> {
    extra
        .iter()
        .cloned()
        .fold::<Vec<ExtraProp>, _>(vec![], |mut extra, (key, value)| {
            match key.as_ref() {
                SKIP_EXTRA_NAME => {
                    if let Some(page_size) = page_size.to_owned() {
                        if extra.iter().all(|(key, _)| key.ne(SKIP_EXTRA_NAME)) {
                            if let Ok(value) = value.parse::<u32>() {
                                let value = (value / page_size as u32) * page_size as u32;
                                extra.push((key, value.to_string()));
                            }
                        }
                    }
                }
                SEARCH_EXTRA_NAME => {
                    if extra.iter().all(|(key, _)| key.ne(SEARCH_EXTRA_NAME)) && !value.is_empty() {
                        extra.push((key, value));
                    }
                }
                _ => {
                    extra.push((key, value));
                }
            };

            extra
        })
}
