use serde::{Deserialize, Serialize};

use crate::ParseError;
/// The priority varying priority levels for a Task.
#[derive(sqlx::Type, Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Asap,
    Far,
}

// Conversion from database i64 to Priority
impl TryFrom<i64> for Priority {
    type Error = ParseError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Priority::Low),
            1 => Ok(Priority::Medium),
            2 => Ok(Priority::High),
            3 => Ok(Priority::Asap),
            4 => Ok(Priority::Far),
            _ => Err(ParseError::FailedToParse),
        }
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
