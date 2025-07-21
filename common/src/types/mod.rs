mod group;
mod id;
mod name;
mod priority;
mod task;

use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
pub use group::*;
pub use id::*;
pub use name::*;
pub use priority::*;
pub use task::*;

use crate::ParseError;

pub fn parse_date_time(possible_date: &str) -> Result<NaiveDateTime, ParseError> {
    let date;

    if let Ok(parsed) = NaiveDateTime::parse_from_str(possible_date, "%m/%d/%Y %H:%M:%S") {
        date = parsed;
    } else if let Ok(parsed) = NaiveDate::parse_from_str(possible_date, "%m/%d/%Y") {
        date = parsed.and_hms_opt(23, 59, 59).unwrap()
    } else if let Ok(parsed) = NaiveDate::parse_from_str(
        format!("{possible_date}/{}", Utc::now().year()).as_str(),
        "%m/%d/%Y",
    ) {
        date = parsed.and_hms_opt(23, 59, 59).unwrap()
    } else {
        return Err(ParseError::FailedToParse);
    }
    Ok(date)
}
