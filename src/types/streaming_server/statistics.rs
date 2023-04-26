use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    name: String,
    path: String,
    length: u64,
    offset: u64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Growler {
    flood: u64,
    pulse: u64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PeerSearch {
    max: u64,
    min: u64,
    sources: Vec<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwarmCap {
    max_speed: u64,
    min_peers: u64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    connections: u64,
    dht: bool,
    growler: Growler,
    handshake_timeout: u64,
    path: String,
    peer_search: PeerSearch,
    swarm_cap: SwarmCap,
    timeout: u64,
    tracker: bool,
    r#virtual: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    last_started: String,
    num_found: u64,
    num_found_uniq: u64,
    num_requests: u64,
    url: Url,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    name: String,
    info_hash: String,
    files: Vec<File>,
    sources: Vec<Source>,
    opts: Options,
    download_speed: f64,
    upload_speed: f64,
    downloaded: u64,
    uploaded: u64,
    unchoked: u64,
    peers: u64,
    queued: u64,
    unique: u64,
    connection_tries: u64,
    peer_search_running: bool,
    stream_len: u64,
    stream_name: String,
    stream_progress: f64,
    swarm_connections: u64,
    swarm_paused: bool,
    swarm_size: u64,
}
