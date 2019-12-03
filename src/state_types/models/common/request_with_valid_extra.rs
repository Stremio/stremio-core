use crate::constants::{CATALOG_PAGE_SIZE, SEARCH, SKIP};
use crate::types::addons::{ExtraProp, ResourceRef, ResourceRequest};

pub fn request_with_valid_extra(request: &ResourceRequest) -> ResourceRequest {
    let extra = request
        .path
        .extra
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
                    if extra.iter().all(|(key, _)| key.ne(SEARCH)) && value.len() > 0 {
                        extra.push((key, value));
                    };
                }
                _ => {
                    extra.push((key, value));
                }
            };

            extra
        });
    ResourceRequest {
        base: request.base.to_owned(),
        path: ResourceRef {
            resource: request.path.resource.to_owned(),
            type_name: request.path.type_name.to_owned(),
            id: request.path.id.to_owned(),
            extra,
        },
    }
}
