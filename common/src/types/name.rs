use serde::{Deserialize, Serialize};
use sqlx::{Database, Decode};

use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Name(String);

impl Deref for Name {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Name {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<String> for Name {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Allows sqlx to decode Name from query results
impl<'r, DB: Database> Decode<'r, DB> for Name
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<Name, Box<dyn Error + 'static + Send + Sync>> {
        let value = <&str as Decode<DB>>::decode(value)?;

        Ok(Name(value.parse()?))
    }
}
