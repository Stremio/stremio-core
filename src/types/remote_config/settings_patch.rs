use serde_json::{Map, Value};

use crate::types::profile::Settings;

const MAX_PATCH_DEPTH: usize = 3;
const MAX_STRING_LENGTH: usize = 64;
const DENIED_SETTINGS: &[&str] = &[
    "streamingServerUrl",
    "streamingServerWarningDismissed",
    "sendCrashReports",
    "playerType",
];

pub fn patch_settings<'a>(
    settings: &Settings,
    patches: impl IntoIterator<Item = &'a Map<String, Value>>,
) -> Option<Settings> {
    let mut value = serde_json::to_value(settings).ok()?;
    for patch in patches {
        if patch
            .keys()
            .any(|key| DENIED_SETTINGS.contains(&key.as_str()))
        {
            return None;
        }
        apply_patch(value.as_object_mut()?, patch, 1)?;
    }
    serde_json::from_value(value).ok()
}

fn apply_patch(
    target: &mut Map<String, Value>,
    patch: &Map<String, Value>,
    depth: usize,
) -> Option<()> {
    if patch.is_empty() || depth > MAX_PATCH_DEPTH {
        return None;
    }
    for (key, next) in patch {
        let current = target.get_mut(key)?;
        match next {
            Value::Object(next) => apply_patch(current.as_object_mut()?, next, depth + 1)?,
            Value::Array(_) => return None,
            _ if current.is_array() || current.is_object() => return None,
            Value::String(string)
                if string.chars().count() > MAX_STRING_LENGTH || string.contains("://") =>
            {
                return None
            }
            _ if !current.is_null() && !next.is_null() && !same_type(current, next) => return None,
            _ => *current = next.to_owned(),
        }
    }
    Some(())
}

fn same_type(a: &Value, b: &Value) -> bool {
    matches!(
        (a, b),
        (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::String(_), Value::String(_))
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Map, Value};

    use super::apply_patch;

    fn object(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().to_owned()
    }

    #[test]
    fn patch_depth_is_limited_to_three_levels() {
        let mut target = object(json!({ "a": { "b": { "c": { "d": 1 } } } }));
        assert!(apply_patch(
            &mut target,
            &object(json!({ "a": { "b": { "c": { "d": 2 } } } })),
            1
        )
        .is_none());
        let mut target = object(json!({ "a": { "b": { "c": 1 } } }));
        assert!(
            apply_patch(&mut target, &object(json!({ "a": { "b": { "c": 2 } } })), 1).is_some()
        );
        assert_eq!(target, object(json!({ "a": { "b": { "c": 2 } } })));
    }
}
