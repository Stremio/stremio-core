use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub name: String,
    pub path: String,
    pub length: u64,
    pub offset: u64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Growler {
    pub flood: u64,
    pub pulse: u64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PeerSearch {
    pub max: u64,
    pub min: u64,
    pub sources: Vec<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwarmCap {
    pub max_speed: u64,
    pub min_peers: u64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub connections: u64,
    pub dht: bool,
    pub growler: Growler,
    pub handshake_timeout: u64,
    pub path: String,
    pub peer_search: PeerSearch,
    pub swarm_cap: SwarmCap,
    pub timeout: u64,
    pub tracker: bool,
    pub r#virtual: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub last_started: String,
    pub num_found: u64,
    pub num_found_uniq: u64,
    pub num_requests: u64,
    pub url: Url,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub name: String,
    pub info_hash: String,
    pub files: Vec<File>,
    pub sources: Vec<Source>,
    pub opts: Options,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub downloaded: u64,
    pub uploaded: u64,
    pub unchoked: u64,
    pub peers: u64,
    pub queued: u64,
    pub unique: u64,
    pub connection_tries: u64,
    pub peer_search_running: bool,
    pub stream_len: u64,
    pub stream_name: String,
    pub stream_progress: f64,
    pub swarm_connections: u64,
    pub swarm_paused: bool,
    pub swarm_size: u64,
}
