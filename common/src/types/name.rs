use crate::ParseError;
use std::ops::{Deref, DerefMut};

#[derive(PartialEq, Eq, Debug)]
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

impl TryFrom<String> for Name {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        //TODO: place parsing here to validate name!
        Ok(Self(value))
    }
}
