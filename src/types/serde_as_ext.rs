use core::marker::PhantomData;
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use std::hash::Hash;

pub trait UniqueVecAdapter<T, U: Eq + Hash> {
    fn hash(value: &T) -> U;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct UniqueVec<T, U, A>(PhantomData<(T, U, A)>);

impl<'de, T, U, A> DeserializeAs<'de, Vec<T>> for UniqueVec<T, U, A>
where
    T: Deserialize<'de>,
    U: Eq + Hash,
    A: UniqueVecAdapter<T, U>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = Vec::<T>::deserialize(deserializer)?;
        Ok(values.into_iter().unique_by(|item| A::hash(item)).collect())
    }
}

impl<T, U, A> SerializeAs<Vec<T>> for UniqueVec<T, U, A>
where
    T: Serialize,
    U: Eq + Hash,
    A: UniqueVecAdapter<T, U>,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        source
            .iter()
            .unique_by(|item| A::hash(item))
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}
