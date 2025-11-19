mod color;
mod group;
mod id;
mod name;
mod priority;
mod task;

pub use color::*;
pub use group::*;
pub use id::*;
pub use name::*;
pub use priority::*;
pub use task::*;

use crate::ParseError;
use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};

//TODO: write tests for this function
pub fn parse_date_time(possible_date: &str) -> Result<NaiveDateTime, ParseError> {
    let formats_datetime = [
        "%-m/%-d/%y %-I:%M%P",  // 12/2/25 9:00pm
        "%-m/%-d/%y %-I:%M %P", // 12/2/25 9:00 pm
        "%m/%d/%Y %H:%M:%S",    // 12/02/2025 21:00:00
        "%-m/%-d/%Y %H:%M",     // 12/2/2025 21:00
    ];

    let formats_date = [
        "%-m/%-d/%y", // 12/2/25
        "%-m/%d/%y",  // 12/2/25
        "%m/%d/%Y",   // 12/02/2025
        "%-m/%-d/%Y", // 12/2/2025
    ];

    // Try datetime formats first
    let mut date = None;
    for format in &formats_datetime {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(possible_date, format) {
            date = Some(parsed);
            break;
        }
    }

    // Try date-only formats (set time to 11:59:59 PM)
    if date.is_none() {
        for format in &formats_date {
            if let Ok(parsed) = NaiveDate::parse_from_str(possible_date, format) {
                date = Some(parsed.and_hms_opt(23, 59, 59).unwrap());
                break;
            }
        }
    }

    // Try with current year appended (for formats like "12/2")
    if date.is_none() {
        let with_year = format!("{}/{}", possible_date, Utc::now().year());
        for format in &["%m/%d/%Y", "%-m/%-d/%Y"] {
            if let Ok(parsed) = NaiveDate::parse_from_str(&with_year, format) {
                date = Some(parsed.and_hms_opt(23, 59, 59).unwrap());
                break;
            }
        }
    }

    date.ok_or(ParseError::Message("Invalid date format".to_string()))
}
