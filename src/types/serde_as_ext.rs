use core::marker::PhantomData;
use itertools::Itertools;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::de::DeserializeAsWrap;
use serde_with::{DeserializeAs, SerializeAs};
use std::fmt;
use std::hash::Hash;

#[derive(Copy, Clone, Debug, Default)]
pub struct Unique<T: Hash + Eq>(PhantomData<T>);

impl<'de, T, U> DeserializeAs<'de, Vec<T>> for Unique<U>
where
    U: Clone + Hash + Eq + Deserialize<'de> + DeserializeAs<'de, T>,
    T: From<U>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(
            transparent,
            bound(deserialize = "DeserializeAsWrap<Vec<T>, Vec<U>>: Deserialize<'de>")
        )]
        struct Helper<'a, T, U>
        where
            U: DeserializeAs<'a, T>,
        {
            #[serde(flatten)]
            inner: DeserializeAsWrap<Vec<T>, Vec<U>>,
            #[serde(skip)]
            marker: PhantomData<&'a u32>,
        }

        impl<'a, T, U> Helper<'a, T, U>
        where
            U: DeserializeAs<'a, T>,
        {
            fn into_inner(self) -> DeserializeAsWrap<Vec<T>, Vec<U>> {
                self.inner
            }
        }

        impl<'de, T, U> Visitor<'de> for Helper<'de, T, U>
        where
            U: Clone + Hash + Eq + Deserialize<'de> + DeserializeAs<'de, T>,
            T: From<U>,
        {
            type Value = Vec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut values = Vec::<U>::with_capacity(visitor.size_hint().unwrap_or_default());
                while let Some(value) = visitor.next_element()? {
                    values.push(value);
                }
                Ok(values.into_iter().unique().map(From::from).collect())
            }
        }

        Ok(Helper::<'de, T, U>::deserialize(deserializer)?
            .into_inner()
            .into_inner())
    }
}

impl<T, U> SerializeAs<Vec<T>> for Unique<U>
where
    U: Clone + Hash + Eq + From<T> + Serialize + SerializeAs<T>,
    T: Clone,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        source
            .iter()
            .cloned()
            .map(From::from)
            .unique()
            .collect::<Vec<U>>()
            .serialize(serializer)
    }
}
