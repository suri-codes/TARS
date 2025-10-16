use std::fmt::Display;

use ratatui::{
    style::Style,
    widgets::{Block, BorderType, Borders},
};
use serde::{Deserialize, Serialize};

use crate::ParseError;
/// The priority varying priority levels for a Task.
#[derive(
    sqlx::Type, Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize, PartialOrd, Ord, Default,
)]
#[repr(i32)]
pub enum Priority {
    Far = 5,
    Low = 4,
    #[default]
    Medium = 3,
    High = 2,
    Asap = 1,
}

impl Priority {
    pub fn parse_clap(str: &str) -> Result<Self, ParseError> {
        str.try_into()
    }
}

impl From<Priority> for Block<'_> {
    fn from(value: Priority) -> Self {
        Block::new()
            .title_top("[P]riority")
            .borders(Borders::all())
            .border_type(BorderType::Rounded)
            .style({
                match value {
                    Priority::Far => Style::new().fg(ratatui::style::Color::LightBlue),
                    Priority::Low => Style::new().fg(ratatui::style::Color::Blue),
                    Priority::Medium => Style::new().fg(ratatui::style::Color::Yellow),
                    Priority::High => Style::new().fg(ratatui::style::Color::LightRed),
                    Priority::Asap => Style::new().fg(ratatui::style::Color::Red),
                }
            })
    }
}
impl TryFrom<&str> for Priority {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Low" => Ok(Priority::Low),
            "low" => Ok(Priority::Low),
            "L" => Ok(Priority::Low),
            "l" => Ok(Priority::Low),

            "Medium" => Ok(Priority::Medium),
            "medium" => Ok(Priority::Medium),
            "M" => Ok(Priority::Medium),
            "m" => Ok(Priority::Medium),

            "High" => Ok(Priority::High),
            "high" => Ok(Priority::High),
            "H" => Ok(Priority::High),
            "h" => Ok(Priority::High),

            "Asap" => Ok(Priority::Asap),
            "asap" => Ok(Priority::Asap),
            "A" => Ok(Priority::Asap),
            "a" => Ok(Priority::Asap),

            "Far" => Ok(Priority::Far),
            "far" => Ok(Priority::Far),
            "F" => Ok(Priority::Far),
            "f" => Ok(Priority::Far),

            _ => Err(ParseError::new(format!(
                "Failed to parse {value} as a valid Priority!"
            ))),
        }
    }
}

impl From<Priority> for String {
    fn from(value: Priority) -> Self {
        match value {
            Priority::Low => "Low".to_owned(),
            Priority::Medium => "Medium".to_owned(),
            Priority::High => "High".to_owned(),
            Priority::Asap => "ASAP".to_owned(),
            Priority::Far => "Far".to_owned(),
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Priority::Low => "Low".to_owned(),
            Priority::Medium => "Medium".to_owned(),
            Priority::High => "High".to_owned(),
            Priority::Asap => "ASAP".to_owned(),
            Priority::Far => "Far".to_owned(),
        };

        write!(f, "{string}")
    }
}
