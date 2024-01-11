
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CalendarItem {
    pub meta_item: MetaItemId,
    pub video_id: VideoId,
    pub video_released: DateTime<Utc>,
}
