use core::marker::PhantomData;
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use std::hash::Hash;

pub trait UniqueByAdapter<T, U: Eq + Hash> {
    fn call(t: &T) -> U;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct UniqueBy<T, U, A>(PhantomData<(T, U, A)>);

impl<'de, T, U, A> DeserializeAs<'de, Vec<T>> for UniqueBy<T, U, A>
where
    T: Deserialize<'de>,
    U: Eq + Hash,
    A: UniqueByAdapter<T, U>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = Vec::<T>::deserialize(deserializer)?;
        Ok(values.into_iter().unique_by(|item| A::call(item)).collect())
    }
}

impl<T, U, A> SerializeAs<Vec<T>> for UniqueBy<T, U, A>
where
    T: Serialize,
    U: Eq + Hash,
    A: UniqueByAdapter<T, U>,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        source
            .iter()
            .unique_by(|item| A::call(item))
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}
