use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to Parse")]
    FailedToParse,
}

#[derive(Error, Debug)]
pub enum TarsError {
    #[error("Reqwest Error!")]
    Reqwest(#[from] reqwest::Error),
}
