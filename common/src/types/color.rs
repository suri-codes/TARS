use rand::random_range;
use ratatui::style::Color as RatColor;
use serde::{Deserialize, Serialize};

use crate::ParseError;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, PartialOrd, Ord)]
/// A wrapper type for colors that can directly be converted to ratatui colors.
pub struct Color(pub String);

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

impl From<Color> for RatColor {
    fn from(value: Color) -> Self {
        let col: RatColor = value.0.parse().unwrap();
        col
    }
}

impl From<RatColor> for Color {
    fn from(value: RatColor) -> Self {
        Color(value.to_string())
    }
}

impl Default for Color {
    fn default() -> Self {
        Self("white".to_owned())
    }
}

impl AsRef<Color> for Color {
    fn as_ref(&self) -> &Color {
        self
    }
}

impl Color {
    /// Parser for clap to form this type from a string.
    pub fn parse_str(str: &str) -> Result<Self, ParseError> {
        let x: Color = str.to_owned().try_into()?;

        Ok(x)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn random() -> Self {
        let rat_col = match random_range(0..=13) {
            0 => RatColor::Red,
            1 => RatColor::Green,
            2 => RatColor::Yellow,
            3 => RatColor::Blue,
            4 => RatColor::Magenta,
            5 => RatColor::Cyan,
            6 => RatColor::Gray,
            7 => RatColor::DarkGray,
            8 => RatColor::LightRed,
            9 => RatColor::LightGreen,
            10 => RatColor::LightYellow,
            11 => RatColor::LightBlue,
            12 => RatColor::LightMagenta,
            13 => RatColor::LightCyan,
            _ => panic!("impossible"),
        };

        rat_col.into()
    }
}
