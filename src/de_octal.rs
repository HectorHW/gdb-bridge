use std::{fmt::Display, str::FromStr};

use serde::Deserialize;
use serde::Deserializer;

pub fn deserialize_number_from_octal_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: serde::Deserialize<'de> + FromStr + num::Unsigned,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => T::from_str_radix(&s, 8)
            .map_err(|_e| "failed to parse octal number")
            .map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}
