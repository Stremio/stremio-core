mod bitfield8;
pub mod watched_bitfield;
pub use watched_bitfield::WatchedBitField;

#[cfg(test)]
mod tests {
    use crate::WatchedBitField;
    #[test]
    fn parse_and_modify() {
        let videos = [
            "tt2934286:1:1",
            "tt2934286:1:2",
            "tt2934286:1:3",
            "tt2934286:1:4",
            "tt2934286:1:5",
            "tt2934286:1:6",
            "tt2934286:1:7",
            "tt2934286:1:8",
            "tt2934286:1:9",
        ];
        let watched = "tt2934286:1:5:5:eJyTZwAAAEAAIA==";
        let mut wb = WatchedBitField::construct_and_resize(
            watched,
            videos.iter().map(|v| v.to_string()).collect(),
        )
        .unwrap();

        assert_eq!(wb.get_video("tt2934286:1:5".to_string()), true);
        assert_eq!(wb.get_video("tt2934286:1:6".to_string()), false);

        assert_eq!(watched, wb.to_string());

        wb.set_video("tt2934286:1:6".to_string(), true);
        assert_eq!(wb.get_video("tt2934286:1:6".to_string()), true);
    }
    #[test]
    fn construct_from_array() {
        let arr = vec![false; 500];
        let mut video_ids = vec![];
        for i in 1..500 {
            video_ids.push(format!("tt2934286:1:{}", i));
        }
        let mut wb = WatchedBitField::construct_from_array(arr, video_ids.clone());

        // All should be false
        for (i, val) in video_ids.iter().enumerate() {
            assert_eq!(wb.get(i), false);
            assert_eq!(wb.get_video(val.to_string()), false);
        }

        // Set half to true
        for (i, _val) in video_ids.iter().enumerate() {
            wb.set(i, i % 2 == 0);
        }

        // Serialize and deserialize to new structure
        let watched = wb.to_string();
        let wb2 = WatchedBitField::construct_and_resize(
            &watched,
            video_ids.iter().map(|v| v.to_string()).collect(),
        )
        .unwrap();

        // Half should still be true
        for (i, val) in video_ids.iter().enumerate() {
            assert_eq!(wb2.get(i), i % 2 == 0);
            assert_eq!(wb2.get_video(val.to_string()), i % 2 == 0);
        }
    }
}
