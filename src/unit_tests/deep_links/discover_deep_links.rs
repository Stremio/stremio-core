use crate::deep_links::DiscoverDeepLinks;
use crate::types::addon::{ResourcePath, ResourceRequest, TransportUrl};

#[test]
fn discover_deep_links() {
    let request = ResourceRequest::new(
        TransportUrl::parse("http://domain.root/manifest.json").unwrap(),
        ResourcePath::without_extra("meta", "movie", "tt1254207"),
    );
    let ddl = DiscoverDeepLinks::try_from(&request).unwrap();
    assert_eq!(
        ddl.discover,
        "stremio:///discover/http%3A%2F%2Fdomain.root%2F/movie/tt1254207?".to_string()
    );
}
