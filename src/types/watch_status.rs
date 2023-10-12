use serde::{Deserialize, Serialize};

use super::addon::ExtraValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// Whether or not the status has been successfully handled by the addon
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Request {
    #[serde(rename_all = "camelCase")]
    Start { current_time: u64, duration: u64 },
    #[serde(rename_all = "camelCase")]
    Resume { current_time: u64, duration: u64 },
    #[serde(rename_all = "camelCase")]
    Pause { current_time: u64, duration: u64 },
    #[serde(rename_all = "camelCase")]
    End { current_time: u64, duration: u64 },
}

impl From<Request> for Vec<ExtraValue> {
    fn from(request: Request) -> Self {
        let (action, current_time, duration) = match request {
            Request::Start {
                current_time,
                duration,
            } => ("start", current_time, duration),
            Request::Resume {
                current_time,
                duration,
            } => ("resume", current_time, duration),
            Request::Pause {
                current_time,
                duration,
            } => ("pause", current_time, duration),
            Request::End {
                current_time,
                duration,
            } => ("end", current_time, duration),
        };

        vec![
            ExtraValue {
                name: "action".to_owned(),
                value: action.to_owned(),
            },
            ExtraValue {
                name: "currentTime".to_owned(),
                value: current_time.to_string(),
            },
            ExtraValue {
                name: "duration".to_owned(),
                value: duration.to_string(),
            },
        ]
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Extra {
//     action: Action,
//     /// milliseconds
//     current_time: u64,
//     /// milliseconds
//     length:
// }
