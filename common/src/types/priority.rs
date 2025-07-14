use ratatui::{
    style::Style,
    widgets::{Block, BorderType, Borders},
};
use serde::{Deserialize, Serialize};

use crate::ParseError;
/// The priority varying priority levels for a Task.
#[derive(
    sqlx::Type, Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize, PartialOrd, Ord,
)]
#[repr(i32)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Asap = 4,
    Far = 5,
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

            _ => Err(ParseError::FailedToParse),
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
