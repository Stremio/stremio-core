use core::{cmp::Ordering, marker::PhantomData};

use std::hash::Hash;

use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{de::DeserializeAsWrap, DeserializeAs, Same, SerializeAs};

pub trait SortedVecAdapter {
    type Input;
    type Args;

    fn args(values: &[Self::Input]) -> Self::Args;
    fn cmp(a: &Self::Input, b: &Self::Input, args: &Self::Args) -> Ordering;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SortedVec<V, A>(PhantomData<(V, A)>);

impl<'de, T, V, A> DeserializeAs<'de, Vec<T>> for SortedVec<V, A>
where
    T: Deserialize<'de>,
    V: DeserializeAs<'de, Vec<T>>,
    A: SortedVecAdapter<Input = T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = V::deserialize_as(deserializer)?;
        let args = A::args(&values);
        Ok(values
            .into_iter()
            .sorted_by(|a, b| A::cmp(a, b, &args))
            .collect())
    }
}

impl<T, V, A> SerializeAs<Vec<T>> for SortedVec<V, A>
where
    T: Clone + Serialize,
    V: SerializeAs<Vec<T>>,
    A: SortedVecAdapter<Input = T>,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let args = A::args(source);

        let source = source
            .iter()
            .sorted_by(|a, b| A::cmp(a, b, &args))
            .cloned()
            .collect::<Vec<_>>();
        V::serialize_as(&source, serializer)
    }
}

pub trait UniqueVecAdapter {
    type Input;
    type Output: Eq + Hash;

    fn hash(value: &Self::Input) -> Self::Output;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct UniqueVec<V, A>(PhantomData<(V, A)>);

impl<'de, T, U, V, A> DeserializeAs<'de, Vec<T>> for UniqueVec<V, A>
where
    T: Deserialize<'de>,
    U: Eq + Hash,
    V: DeserializeAs<'de, Vec<T>>,
    A: UniqueVecAdapter<Input = T, Output = U>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = V::deserialize_as(deserializer)?;
        Ok(values.into_iter().unique_by(|item| A::hash(item)).collect())
    }
}

impl<T, U, V, A> SerializeAs<Vec<T>> for UniqueVec<V, A>
where
    T: Clone + Serialize,
    U: Eq + Hash,
    V: SerializeAs<Vec<T>>,
    A: UniqueVecAdapter<Input = T, Output = U>,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let source = source
            .iter()
            .unique_by(|item| A::hash(item))
            .cloned()
            .collect::<Vec<_>>();
        V::serialize_as(&source, serializer)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct NumberAsString;

impl<'de> DeserializeAs<'de, String> for NumberAsString {
    fn deserialize_as<D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            Number(f64),
            String(String),
        }

        Ok(match Helper::deserialize(deserializer)? {
            Helper::Number(value) => value.to_string(),
            Helper::String(value) => value,
        })
    }
}

/// Deserialize an Option from a [`bool`] or the underlying `Option`.
/// For both `true` and `false` values, the Option will be set to `None`
///
/// # Examples
/// ```
/// use serde::Deserialize;
/// use serde_with::serde_as;
/// use stremio_core::types::DefaultOnBool;
///
/// #[serde_as]
/// #[derive(Deserialize, Debug, PartialEq, Eq)]
/// struct MyType {
///     #[serde_as(deserialize_as = "DefaultOnBool")]
///     x: Option<u32>,
/// }
///
/// let json = serde_json::json!({ "x": false });
/// assert_eq!(MyType { x: None }, serde_json::from_value::<MyType>(json).expect("Should deserialize"));
///
/// let json = serde_json::json!({ "x": true });
/// assert_eq!(MyType { x: None }, serde_json::from_value::<MyType>(json).expect("Should deserialize"));
///
/// let json = serde_json::json!({ "x": null });
/// assert_eq!(MyType { x: None }, serde_json::from_value::<MyType>(json).expect("Should deserialize"));
///
/// let json = serde_json::json!({ "x": 32 });
/// assert_eq!(MyType { x: Some(32) }, serde_json::from_value::<MyType>(json).expect("Should deserialize"));
/// ```
#[derive(Copy, Clone, Debug)]
pub struct DefaultOnBool<T = Same>(PhantomData<T>);

impl<'de, T, U> DeserializeAs<'de, T> for DefaultOnBool<U>
where
    U: DeserializeAs<'de, T>,
    T: Default,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(
            match BoolOrValue::<DeserializeAsWrap<T, U>>::deserialize(deserializer)? {
                BoolOrValue::Bool(_bool) => T::default(),
                BoolOrValue::Value(value) => value.into_inner(),
            },
        )
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BoolOrValue<T> {
    Bool(bool),
    Value(T),
}
#[cfg(test)]
mod tests {
    use super::DefaultOnBool;

    use serde::Deserialize;
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct MyType {
        #[serde_as(deserialize_as = "DefaultOnBool")]
        pub x: Option<u32>,
    }

    #[test]
    fn test_bool_as_option() {
        let json = serde_json::json!({ "x": null });
        assert_eq!(
            MyType { x: None },
            serde_json::from_value::<MyType>(json).expect("Should deserialize")
        );

        let json = serde_json::json!({ "x": false });
        assert_eq!(
            MyType { x: None },
            serde_json::from_value::<MyType>(json).expect("Should deserialize")
        );

        let json = serde_json::json!({ "x": true });
        assert_eq!(
            MyType { x: None },
            serde_json::from_value::<MyType>(json).expect("Should deserialize")
        );

        let json = serde_json::json!({ "x": 32 });
        assert_eq!(
            MyType { x: Some(32) },
            serde_json::from_value::<MyType>(json).expect("Should deserialize")
        );
    }
}
