use chrono::{TimeZone, Utc};

use crate::{
    constants::CATALOG_RESOURCE_NAME,
    models::{
        calendar::{Calendar, FullDate, MonthInfo, YearMonthDate},
        common::{Loadable, ResourceLoadable},
        ctx::Ctx,
    },
    runtime::{
        msg::{Internal, Msg},
        UpdateWithCtx,
    },
    types::{
        addon::{ResourcePath, ResourceRequest, ResourceResponse},
        resource::{MetaItem, MetaItemPreview, Video},
    },
    unit_tests::TestEnv,
};

fn meta_item(id: &str, released: &[(i32, u32, u32)]) -> MetaItem {
    MetaItem {
        preview: MetaItemPreview {
            id: id.to_owned(),
            r#type: "series".to_owned(),
            ..MetaItemPreview::default()
        },
        videos: released
            .iter()
            .enumerate()
            .map(|(index, (year, month, day))| Video {
                id: format!("{id}:{index}"),
                released: Some(Utc.with_ymd_and_hms(*year, *month, *day, 0, 0, 0).unwrap()),
                ..Video::default()
            })
            .chain([Video {
                id: format!("{id}:unreleased"),
                ..Video::default()
            }])
            .collect(),
    }
}

#[test]
fn calendar_items_group_videos_by_release_day() {
    let _guard = TestEnv::reset().unwrap();
    let request = ResourceRequest {
        base: "https://addon/manifest.json".parse().unwrap(),
        path: ResourcePath {
            resource: CATALOG_RESOURCE_NAME.to_owned(),
            r#type: "series".to_owned(),
            id: "calendarVideosIds".to_owned(),
            extra: vec![],
        },
    };
    let mut calendar = Calendar {
        selected: Some(YearMonthDate {
            month: 1,
            year: 2024,
        }),
        month_info: MonthInfo {
            today: None,
            days: 31,
            first_weekday: 0,
        },
        meta_items: vec![ResourceLoadable {
            request: request.clone(),
            content: Some(Loadable::Loading),
        }],
        ..Default::default()
    };
    let result = Ok(ResourceResponse::MetasDetailed {
        metas_detailed: vec![
            meta_item(
                "tt1",
                &[(2024, 1, 15), (2024, 1, 31), (2024, 2, 1), (2023, 12, 15)],
            ),
            meta_item("tt2", &[(2024, 1, 15), (2025, 1, 1)]),
        ],
    });

    <Calendar as UpdateWithCtx<TestEnv>>::update(
        &mut calendar,
        &Msg::Internal(Internal::ResourceRequestResult(request, Box::new(result))),
        &Ctx::default(),
    );

    assert_eq!(calendar.items.len(), 31);
    assert_eq!(calendar.items[30].date, FullDate::new(2024, 1, 31));
    let days_with_videos = calendar
        .items
        .iter()
        .filter(|item| !item.items.is_empty())
        .map(|item| {
            (
                item.date.day,
                item.items
                    .iter()
                    .map(|content| content.video.id.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        days_with_videos,
        vec![(15, vec!["tt1:0", "tt2:0"]), (31, vec!["tt1:1"])]
    );
}
