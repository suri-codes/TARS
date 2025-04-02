use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to Parse")]
    FailedToParse,
}
