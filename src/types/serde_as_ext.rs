use core::marker::PhantomData;
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use std::hash::Hash;

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
