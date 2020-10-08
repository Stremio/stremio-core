use crate::types::addon::ResourceResponse;
use crate::types::resource::MetaItem;

#[test]
fn deserialize_resource_response_metas() {
    let metas = ResourceResponse::Metas { metas: vec![] };
    let metas_json = r#"
    {
        "metas": []
    }
    "#;
    let metas_deserialize = serde_json::from_str(&metas_json).unwrap();
    assert_eq!(metas, metas_deserialize, "Metas deserialized successfully");
}

#[test]
fn deserialize_resource_response_metas_detailed() {
    let metas_detailed = ResourceResponse::MetasDetailed {
        metas_detailed: vec![],
    };
    let metas_detailed_json = r#"
    {
        "metasDetailed": []
    }
    "#;
    let metas_detailed_deserialize = serde_json::from_str(&metas_detailed_json).unwrap();
    assert_eq!(
        metas_detailed, metas_detailed_deserialize,
        "MetasDetailed deserialized successfully"
    );
}

#[test]
fn deserialize_resource_response_meta() {
    let meta = ResourceResponse::Meta {
        meta: MetaItem::default(),
    };
    let meta_json = format!(
        r#"{{ "meta": {} }}"#,
        serde_json::to_string(&MetaItem::default()).unwrap()
    );
    let meta_deserialize = serde_json::from_str(&meta_json).unwrap();
    assert_eq!(meta, meta_deserialize, "Meta deserialized successfully");
}

#[test]
fn deserialize_resource_response_streams() {
    let streams = ResourceResponse::Streams { streams: vec![] };
    let streams_json = r#"
    {
        "streams": []
    }
    "#;
    let streams_deserialize = serde_json::from_str(&streams_json).unwrap();
    assert_eq!(
        streams, streams_deserialize,
        "Streams deserialized successfully"
    );
}

#[test]
fn deserialize_resource_response_subtitles() {
    let subtitles = ResourceResponse::Subtitles { subtitles: vec![] };
    let subtitles_json = r#"
    {
        "subtitles": []
    }
    "#;
    let subtitles_deserialize = serde_json::from_str(&subtitles_json).unwrap();
    assert_eq!(
        subtitles, subtitles_deserialize,
        "Subtitles deserialized successfully"
    );
}

#[test]
fn deserialize_resource_response_addons() {
    let addons = ResourceResponse::Addons { addons: vec![] };
    let addons_json = r#"
    {
        "addons": []
    }
    "#;
    let addons_deserialize = serde_json::from_str(&addons_json).unwrap();
    assert_eq!(
        addons, addons_deserialize,
        "Addons deserialized successfully"
    );
}
