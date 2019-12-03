use crate::constants::{CATALOG_PAGE_SIZE, SEARCH, SKIP};
use crate::types::addons::ExtraProp;

pub fn validate_extra(extra: &[ExtraProp]) -> Vec<ExtraProp> {
    extra
        .iter()
        .cloned()
        .fold::<Vec<ExtraProp>, _>(vec![], |mut extra, (key, value)| {
            match key.as_ref() {
                SKIP => {
                    if extra.iter().all(|(key, _)| key.ne(SKIP)) {
                        if let Ok(value) = value.parse::<u32>() {
                            let value =
                                (value / CATALOG_PAGE_SIZE as u32) * CATALOG_PAGE_SIZE as u32;
                            extra.push((key, value.to_string()));
                        };
                    };
                }
                SEARCH => {
                    if extra.iter().all(|(key, _)| key.ne(SEARCH)) && value.is_empty() {
                        extra.push((key, value));
                    };
                }
                _ => {
                    extra.push((key, value));
                }
            };

            extra
        })
}
