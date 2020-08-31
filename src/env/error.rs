use std::error::Error;
use std::fmt;
use wasm_bindgen::prelude::JsValue;
use wasm_bindgen::JsCast;

#[derive(Debug)]
pub enum EnvError {
    Js(String),
    Serde(serde_json::error::Error),
    HTTPStatusCode(u16),
    StorageMissing,
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for EnvError {
    fn description(&self) -> &str {
        match self {
            EnvError::Js(message) => &message,
            #[allow(deprecated)]
            EnvError::Serde(error) => &error.description(),
            EnvError::HTTPStatusCode(_) => "Unexpected HTTP status code",
            EnvError::StorageMissing => "localStorage is missing",
        }
    }
}

impl From<JsValue> for EnvError {
    fn from(error: JsValue) -> EnvError {
        EnvError::Js(
            error
                .dyn_into::<js_sys::Error>()
                .map(|error| error.to_string().into())
                .unwrap_or("Unknown".to_owned()),
        )
    }
}

impl From<serde_json::error::Error> for EnvError {
    fn from(error: serde_json::error::Error) -> EnvError {
        EnvError::Serde(error)
    }
}
