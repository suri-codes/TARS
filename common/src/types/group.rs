use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

use sqlx::{Database, Decode};

use crate::ParseError;

#[derive(PartialEq, Eq, Debug)]
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
impl TryFrom<&str> for Group {
    type Error = ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        //TODO: place parsing here to validate group!
        Ok(Self(value.to_owned()))
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
