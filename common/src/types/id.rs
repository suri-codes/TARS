use nanoid::nanoid;
use std::ops::{Deref, DerefMut};

use crate::ParseError;

#[derive(PartialEq, Eq, Debug)]
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

impl TryFrom<String> for Id {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
