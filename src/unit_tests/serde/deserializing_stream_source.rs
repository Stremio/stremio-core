use crate::types::resource::StreamSource;
use url::Url;

#[test]
fn deserialize_stream_source_url() {
    let url = StreamSource::Url {
        url: Url::parse("https://url").unwrap(),
    };
    let url_json = r#"
    {
        "url": "https://url/"
    }
    "#;
    let url_deserialize = serde_json::from_str(&url_json).unwrap();
    assert_eq!(url, url_deserialize, "Url deserialized successfully");
}

#[test]
fn deserialize_stream_source_youtube() {
    let youtube = StreamSource::YouTube {
        yt_id: "yt_id".to_owned(),
    };
    let youtube_json = r#"
    {
        "ytId": "yt_id"
    }
    "#;
    let youtube_deserialize = serde_json::from_str(&youtube_json).unwrap();
    assert_eq!(
        youtube, youtube_deserialize,
        "YouTube deserialized successfully"
    );
}

#[test]
fn deserialize_stream_source_torrent() {
    let torrents = vec![
        StreamSource::Torrent {
            info_hash: [1; 20],
            file_idx: Some(1),
        },
        StreamSource::Torrent {
            info_hash: [1; 20],
            file_idx: None,
        },
    ];
    let torrents_json = r#"
    [
        {
            "infoHash": "0101010101010101010101010101010101010101",
            "fileIdx": 1
        },
        {
            "infoHash": "0101010101010101010101010101010101010101",
            "fileIdx": null
        }
    ]
    "#;
    let torrents_deserialize: Vec<StreamSource> = serde_json::from_str(&torrents_json).unwrap();
    assert_eq!(
        torrents, torrents_deserialize,
        "Torrent deserialized successfully"
    );
}
