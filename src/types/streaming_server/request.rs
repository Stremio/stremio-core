use http::{header::CONTENT_TYPE, Request};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{models::streaming_server::StatisticsRequest, types::resource::InfoHash};

pub struct ArchiveStreamRequest {
    /// The `rar/create` or `zip/create` key returned in the response
    pub response_key: String,
    pub options: ArchiveStreamOptions,
}

impl ArchiveStreamRequest {
    pub fn to_query_pairs(self) -> Vec<(String, String)> {
        let options = serde_json::to_value(&self.options).expect("should serialize");
        let options_object = options.as_object().expect("Should be an object");

        vec![
            (
                "key".into(),
                // append the length of the options
                // keep in mind that `None` options should always be treated as not-set
                // i.e. should not be serialized
                format!(
                    "{key}{length}",
                    key = self.response_key,
                    length = options_object.len()
                ),
            ),
            ("o".into(), options.to_string()),
        ]
    }
}

/// Server's `rar/stream` and `zip/stream` options of the query.
///
/// Format: `rar/stream?key={create_key}{options length}&o={options_json_string}`
///
/// Where all parameters are url encoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_idx: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_must_include: Vec<String>,
}

/// # Examples
///
/// Example which creates a request url with no body for the server:
/// `http://127.0.0.1:11470/opensubHash?videoUrl=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4`
///
/// ```
/// use core::types::streaming_server::CreateTorrentRequest;
///
/// let request: http::Request = CreateTorrentRequest {
///     server_url: "http://127.0.0.1:11470/".parse().unwrap(),
///     sources: vec!["https://example.com/my-awesome-video.mp4".parse().unwrap()]
///     info_hash: "".parse().unwrap()
///     file_idx: 1,
///     
/// }.into();
///
/// assert_eq!("http://127.0.0.1:11470/opensubHash?videoUrl=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4", request.uri().to_string());
/// assert_eq!(&(), request.body());
/// ```
pub struct CreateTorrentRequest {
    pub server_url: Url,
    pub sources: Vec<Url>,
    pub info_hash: InfoHash,
    pub file_idx: u64,
}

impl Into<Request<()>> for CreateTorrentRequest {
    fn into(self) -> Request<()> {
        // var query = Array.isArray(sources) && sources.length > 0 ?
        //     '?' + new URLSearchParams(sources.map(function(source) {
        //         return ['tr', source];
        //     }))
        //     :
        //     '';
        // return {
        //     url: url.resolve(streamingServerURL, '/' + encodeURIComponent(infoHash) + '/' + encodeURIComponent(fileIdx)) + query,
        //     infoHash: infoHash,
        //     fileIdx: fileIdx,
        //     sources: sources
        // }
        let url = {
            let mut uri = self
                .server_url
                .join(&format!("{}/{}", self.info_hash, self.file_idx))
                .expect("Should always be valid Url");

            {
                let mut x = uri.query_pairs_mut();
                if !self.sources.is_empty() {
                    for source in self.sources {
                        x.append_pair("tr", source.as_str());
                    }
                }
            }

            // x.finish();
            uri
        };

        Request::builder()
            .uri(url.as_str())
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .body(())
            .expect("Should always be valid Request!")
    }
}

/// # Examples
///
/// Example which creates a request url with no body for the server:
/// `http://127.0.0.1:11470/opensubHash?videoUrl=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4`
///
/// ```
/// use core::types::streaming_server::OpensubtitlesParapRequest;
///
/// let request: http::Request = OpensubtitlesParapRequest {
///     server_url: "http://127.0.0.1:11470/".parse().unwrap()
///     media_url: "https://example.com/my-awesome-video.mp4".parse().unwrap()
///     
/// }.into();
///
/// assert_eq!("http://127.0.0.1:11470/opensubHash?videoUrl=https%3A%2F%2Fexample.com%2Fmy-awesome-video.mp4", request.uri().to_string());
/// assert_eq!(&(), request.body());
/// ```
pub struct OpensubtitlesParamsRequest {
    pub server_url: Url,
    pub media_url: Url,
}

impl Into<Request<()>> for OpensubtitlesParamsRequest {
    fn into(self) -> Request<()> {
        let url = {
            let mut uri = self
                .server_url
                .join("opensubHash")
                .expect("Should always be valid Url");
            {
                let mut x = uri.query_pairs_mut();

                x.append_pair("videoUrl", self.media_url.as_str());
            }

            // x.finish();
            uri
        };

        Request::builder()
            .uri(url.as_str())
            .body(())
            .expect("Should always be valid Request!")
    }
}

/// Filename request to the server.
///
/// `{streaming_sever_url}/{info_hash_url_encoded}/{file_idx_url_encoded/stats.json`
///
///
/// Example: `http://127.0.0.1:11470/6d0cdb871b81477d00f53f78529028994b364877/7/stats.json`
pub struct FileNameRequest {
    pub server_url: Url,
    pub request: StatisticsRequest,
    // pub info_hash: InfoHash,
    // pub file_idx: u64,
}

impl Into<Request<()>> for FileNameRequest {
    fn into(self) -> Request<()> {
        let mut info_hash_encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_key_only(&self.request.info_hash.to_string())
            .finish();
        let mut file_idx_encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_key_only(&self.request.file_idx.to_string())
            .finish();

        let uri = self
            .server_url
            .join(&format!(
                "{info_hash_encoded}/{file_idx_encoded}/stats.json"
            ))
            .expect("Should always be valid url!");

        Request::builder()
            .uri(uri.to_string())
            .body(())
            .expect("Always valid request!")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_to_serde_json_value_keys_length() {
        // 0 keys
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: None,
                file_must_include: vec![],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert!(object.is_empty());
        }

        // only fileIdx
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: Some(1),
                file_must_include: vec![],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert_eq!(1, object.len(), "Only fileIdx is set");
            assert_eq!(object.keys().next().cloned(), Some("fileIdx".to_string()));
        }

        // both keys are set
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: Some(1),
                file_must_include: vec!["fileName".into(), "nameFile".into()],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert_eq!(2, object.len(), "Only fileIdx is set");
        }
    }
}
