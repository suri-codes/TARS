use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};
use sqlx::{Database, Decode};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Group(String);

impl Deref for Group {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Group {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&str> for Group {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for Group {
    fn from(value: String) -> Self {
        Self(value)
    }
}

// DB is the database driver
// `'r` is the lifetime of the `Row` being decoded
impl<'r, DB: Database> Decode<'r, DB> for Group
where
    // we want to delegate some of the work to string decoding so let's make sure strings
    // are supported by the database
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<Group, Box<dyn Error + 'static + Send + Sync>> {
        // the interface of ValueRef is largely unstable at the moment
        // so this is not directly implementable

        // however, you can delegate to a type that matches the format of the type you want
        // to decode (such as a UTF-8 string)

        let value = <&str as Decode<DB>>::decode(value)?;

        // now you can parse this into your type (assuming there is a `FromStr`)

        Ok(Group(value.parse()?))
    }
}
