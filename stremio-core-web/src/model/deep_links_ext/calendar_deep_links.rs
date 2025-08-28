use crate::model::deep_links_ext::DeepLinksExt;
use stremio_core::deep_links::{CalendarDeepLinks, CalendarItemDeepLinks};

impl DeepLinksExt for CalendarDeepLinks {
    fn into_web_deep_links(self) -> Self {
        Self {
            calendar: self.calendar.replace("stremio://", "#"),
        }
    }
}

impl DeepLinksExt for CalendarItemDeepLinks {
    fn into_web_deep_links(self) -> Self {
        Self {
            meta_details_streams: self.meta_details_streams.replace("stremio://", "#"),
        }
    }
}
