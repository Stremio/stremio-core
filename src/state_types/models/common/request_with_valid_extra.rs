use crate::types::addons::{ExtraProp, ResourceRef, ResourceRequest};

pub fn request_with_valid_extra(request: &ResourceRequest, page_size: usize) -> ResourceRequest {
    let extra = request
        .path
        .extra
        .iter()
        .cloned()
        .fold::<Vec<ExtraProp>, _>(vec![], |mut extra, (key, value)| {
            match key.as_ref() {
                "skip" => {
                    if extra.iter().all(|(key, _)| key.ne("skip")) {
                        if let Ok(value) = value.parse::<u32>() {
                            let value = (value / page_size as u32) * page_size as u32;
                            extra.push((key, value.to_string()));
                        };
                    };
                }
                "search" => {
                    if extra.iter().all(|(key, _)| key.ne("search")) && value.len() > 0 {
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
