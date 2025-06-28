use serde::{Deserialize, Serialize};
use sqlx::Decode;
use sqlx::{Database, Encode, Sqlite, Type};
use std::fmt::Display;

use color_eyre::owo_colors::OwoColorize;
use ratatui::style::Color as RatColor;
use tracing::error;

use std::error::Error;

use crate::{ParseError, TarsClient, TarsError};

use super::{Id, Name};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, PartialOrd, Ord)]
/// A group is what represents a collection of tasks or other groups that share some property.
pub struct Group {
    pub id: Id,
    pub name: Name,
    pub parent_id: Option<Id>,
    pub color: Color,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, PartialOrd, Ord)]
/// A wrapper type for colors that can directly be converted to ratatui colors.
pub struct Color(String);

impl TryFrom<String> for Color {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let _col: RatColor = value.parse().map_err(|_| ParseError::FailedToParse)?;

        Ok(Self(value))
    }
}

impl From<&Color> for RatColor {
    fn from(value: &Color) -> Self {
        let col: RatColor = value.0.parse().unwrap();
        col
    }
}

impl Default for Color {
    fn default() -> Self {
        Self("white".to_owned())
    }
}

impl Color {
    /// Parser for clap to form this type from a string.
    pub fn parse_clap(arg: &str) -> Result<Self, ParseError> {
        let x: Color = arg.to_owned().try_into()?;

        Ok(x)
    }
}
impl Group {
    pub fn with_all_fields(
        id: impl Into<Id>,
        name: impl Into<Name>,
        parent_id: Option<Id>,
        color: Color,
    ) -> Self {
        Group {
            id: id.into(),
            name: name.into(),
            parent_id,
            color,
        }
    }

    /// Creates a new `Group` through the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn new(
        client: &TarsClient,
        name: impl Into<Name>,
        parent_id: Option<Id>,
        color: Color,
    ) -> Result<Self, TarsError> {
        let group = Group::with_all_fields(Id::default(), name, parent_id, color);

        let res: Group = client
            .conn
            .post(client.base_path.join("/group/create")?)
            .json(&group)
            .send()
            .await
            .inspect_err(|e| error!("Error creating Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error creating Group: {:?}", e))?;

        Ok(res)
    }

    /// Fetches all `Group`s.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn fetch_all(client: &TarsClient) -> Result<Vec<Group>, TarsError> {
        let res: Vec<Group> = client
            .conn
            .get(client.base_path.join("/group")?)
            .send()
            .await
            .inspect_err(|e| error!("Error Fetching Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error Fetching Group: {:?}", e))?;

        Ok(res)
    }

    /// Sync's this `Group` with its representation in database, via the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// + Something goes wrong with the requests to the Daemon.
    /// + Will panic at runtime if the sync'd `Group` doesnt match with `self`
    pub async fn sync(&self, client: &TarsClient) -> Result<(), TarsError> {
        let res: Group = client
            .conn
            .post(client.base_path.join("/group/update")?)
            .json(self)
            .send()
            .await
            .inspect_err(|e| error!("Error Sync'ing Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error Sync'ing Group: {:?}", e))?;

        assert_eq!(res, *self);

        Ok(())
    }

    /// Deletes this `Group` via the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// + Something goes wrong with the requests to the Daemon.
    /// + Will panic at runtime if deleted `Group` doesnt match the `Group` we wanted to delete.
    pub async fn delete(self, client: &TarsClient) -> Result<(), TarsError> {
        let deleted: Group = client
            .conn
            .post(client.base_path.join("/group/delete")?)
            .json(&self)
            .send()
            .await
            .inspect_err(|e| error!("Error Deleting Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error Deleting Group: {:?}", e))?;

        assert_eq!(deleted, self);

        Ok(())
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", (*self.name).green())?;
        write!(f, "Id: {}", *self.id)?;
        if let Some(ref parent_id) = self.parent_id {
            write!(f, "\nParent Id: {}", **parent_id)?;
        }

        Ok(())
    }
}

// DB is the database driver
// `'r` is the lifetime of the `Row` being decoded
impl<'r, DB: Database> Decode<'r, DB> for Color
where
    // we want to delegate some of the work to string decoding so let's make sure strings
    // are supported by the database
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<Color, Box<dyn Error + 'static + Send + Sync>> {
        // the interface of ValueRef is largely unstable at the moment
        // so this is not directly implementable

        // however, you can delegate to a type that matches the format of the type you want
        // to decode (such as a UTF-8 string)

        let value = <&str as Decode<DB>>::decode(value)?;

        // now you can parse this into your type (assuming there is a `FromStr`)

        Ok(Color(value.parse()?))
    }
}

impl<'q> Encode<'q, Sqlite> for Color {
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

impl Type<Sqlite> for Color {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        todo!()
    }
}
