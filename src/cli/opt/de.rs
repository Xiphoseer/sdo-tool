use std::path::PathBuf;

use serde::{
    de::{Error, Visitor},
    Deserializer,
};

pub(super) fn deserialize_opt_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_string(OptStringVisitor)
}

struct OptStringVisitor;

impl<'de> Visitor<'de> for OptStringVisitor {
    type Value = Option<String>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Some(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Some(v.to_owned()))
    }
}

pub(super) fn _deserialize_opt_path<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_string(_OptPathVisitor)
}

struct _OptPathVisitor;

impl<'de> Visitor<'de> for _OptPathVisitor {
    type Value = Option<PathBuf>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Some(PathBuf::from(v)))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Some(PathBuf::from(v.to_owned())))
    }
}

pub(super) fn deserialize_opt_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_i32(OptI32Visitor)
}

struct OptI32Visitor;

impl<'de> Visitor<'de> for OptI32Visitor {
    type Value = Option<i32>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a number")
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Some(v))
    }
}
