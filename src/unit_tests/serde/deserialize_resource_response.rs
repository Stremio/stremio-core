use crate::types::addon::ResourceResponse::Metas;
use crate::types::resource::{MetaItemBehaviorHints, MetaItemPreview};

#[test]
fn deserialize_resource_response_metas() {
    let metas_vec = vec![MetaItemPreview {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        trailers: vec![],
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: None,
            featured_video_id: None,
        },
    }];
    let metas_json = r#"{"metas":[{"id":"id","type":"type_name","name":"name","poster":null,"logo":null,"description":null,"releaseInfo":null,"runtime":null,"released":null,"posterShape":"poster","trailers":[],"behaviorHints":{"defaultVideoId":null,"featuredVideoId":null}}]}"#;
    let metas_deserialize = serde_json::from_str(&metas_json).unwrap();
    match metas_deserialize {
        Metas { metas } => assert_eq!(metas, metas_vec, "metas deserialized successfully"),
        _ => panic!("failed getting metas"),
    };
}
