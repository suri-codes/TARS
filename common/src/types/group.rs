use std::ops::{Deref, DerefMut};

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
impl TryFrom<String> for Group {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        //TODO: place parsing here to validate group!
        Ok(Self(value))
    }
}
