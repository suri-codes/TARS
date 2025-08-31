use axum::response::IntoResponse;
use reqwest::StatusCode;
use thiserror::Error;
use tokio::sync::broadcast::error::SendError;

use crate::Diff;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to Parse")]
    FailedToParse,
}

#[derive(Error, Debug)]
pub enum TarsError {
    #[error("Reqwest Error!")]
    Reqwest(#[from] reqwest::Error),

    #[error("Sqlx Error!")]
    Sqlx(#[from] sqlx::Error),

    #[error("Conversion Error!")]
    Parse(#[from] ParseError),

    #[error("Url Error!")]
    UrlError(#[from] url::ParseError),

    #[error("Send Error!")]
    SendError(#[from] SendError<Diff>),
}

impl IntoResponse for TarsError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            TarsError::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TarsError::Sqlx(ref e) => match e {
                sqlx::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
                sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            TarsError::Parse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // this would never be hit
            TarsError::UrlError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TarsError::SendError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        tracing::error!("TarsError: {:?}, returning status code: {}", self, status);

        status.into_response()
    }
}
