use crate::deep_links::SearchHistoryItemDeepLinks;

#[test]
fn search_history_item_deep_links() {
    let query = "better call saul".to_string();
    let deep_link = SearchHistoryItemDeepLinks::from(&query);
    assert_eq!(
        deep_link.search,
        "stremio:///search?query=better%20call%20saul".to_string(),
    );
}
