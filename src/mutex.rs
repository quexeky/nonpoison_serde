use std::{ops::Deref, sync::nonpoison};

use serde::{Deserialize, Serialize, de::Visitor, ser::SerializeStruct};

#[derive(Debug)]
pub struct Mutex<T: Serialize + for<'de> Deserialize<'de>>(nonpoison::Mutex<T>);

impl<T: Serialize + for<'de> Deserialize<'de>> Mutex<T> {
    pub fn new(inner: T) -> Self {
        Mutex(nonpoison::Mutex::new(inner))
    }
}
impl<T: Serialize + for<'de> Deserialize<'de> + Into<nonpoison::Mutex<T>>> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Mutex(nonpoison::Mutex::from(value))
    }
}
impl<T: Serialize + for<'de> Deserialize<'de>> From<nonpoison::Mutex<T>> for Mutex<T> {
    fn from(value: nonpoison::Mutex<T>) -> Self {
        Mutex(value)
    }
}

impl<T: Serialize + for<'de> Deserialize<'de>> Deref for Mutex<T> {
    type Target = nonpoison::Mutex<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Serialize + for<'de> Deserialize<'de>> Serialize for Mutex<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Mutex", 1)?;
        state.serialize_field("inner", &*self.0.lock())?;
        state.end()
    }
}
impl<'a, T: Serialize + for<'de> Deserialize<'de>> Deserialize<'a> for Mutex<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Inner,
        }

        struct MutexVisitor<T> {
            marker: std::marker::PhantomData<T>,
        }

        impl<'de, T: Serialize + for<'b> Deserialize<'b>> Visitor<'de> for MutexVisitor<T> {
            type Value = Mutex<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Mutex with field 'inner'")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut inner: Option<T> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Inner => {
                            if inner.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner = Some(map.next_value()?);
                        }
                    }
                }

                let inner = inner.ok_or_else(|| serde::de::Error::missing_field("inner"))?;
                Ok(Mutex::new(inner))
            }
        }
        deserializer.deserialize_struct(
            "Mutex",
            &["inner"],
            MutexVisitor {
                marker: std::marker::PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, rng};
    use serde::{Deserialize, Serialize};

    use crate::Mutex;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
    struct Dummy {
        inner: String,
    }
    #[test]
    fn serde() {
        let data = rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let dummy = Dummy { inner: data };

        let to_serialize = Mutex::new(dummy.clone());
        let serialized = serde_json::to_string(&to_serialize).unwrap();
        assert_eq!(
            *to_serialize.lock(),
            *serde_json::from_str::<Mutex<Dummy>>(&serialized)
                .unwrap()
                .lock()
        );
    }
}
