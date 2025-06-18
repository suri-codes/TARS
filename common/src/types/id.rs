use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Decode, Encode, Sqlite, Type};
use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

/// holds an Id used in all the types stored in the Database.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Id(String);

impl Deref for Id {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Id {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Returns a random Id
impl Default for Id {
    fn default() -> Self {
        Id(nanoid!())
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Self(value)
    }
}

// DB is the database driver
// `'r` is the lifetime of the `Row` being decoded
impl<'r, DB: Database> Decode<'r, DB> for Id
where
    // we want to delegate some of the work to string decoding so let's make sure strings
    // are supported by the database
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<Id, Box<dyn Error + 'static + Send + Sync>> {
        // the interface of ValueRef is largely unstable at the moment
        // so this is not directly implementable

        // however, you can delegate to a type that matches the format of the type you want
        // to decode (such as a UTF-8 string)

        let value = <&str as Decode<DB>>::decode(value)?;

        // now you can parse this into your type (assuming there is a `FromStr`)

        Ok(Id(value.parse()?))
    }
}

impl<'q> Encode<'q, Sqlite> for Id {
    fn encode(
        self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        <std::string::String as sqlx::Encode<'_, Sqlite>>::encode(self.0, buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <std::string::String as sqlx::Encode<'_, Sqlite>>::encode_by_ref(&self.0, buf)
    }
}

impl Type<Sqlite> for Id {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        todo!()
    }
}
