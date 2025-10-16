use std::str::FromStr;

use reqwest::{Client, ClientBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::{
    TarsResult,
    types::{Group, Id, Task},
};
/// Holds the reqwest `Client` and the base path for accessing the `TarsDaemon`
#[derive(Debug, Clone)]
pub struct TarsClient {
    pub base_path: Url,
    pub conn: Client,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Diff {
    Added(DiffInner),
    Updated(DiffInner),
    Deleted(Id),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffInner {
    Task(Task),
    Group(Group),
}

impl TarsClient {
    /// Creates a new TarsClient with the provided base_url.
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    /// + Connecting to the daemon fails.
    pub async fn new(base_url: String) -> TarsResult<Self> {
        let app_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let client = ClientBuilder::new().user_agent(app_agent).build()?;

        Ok(Self {
            conn: client,
            base_path: Url::from_str(&base_url).expect("should be a valid url"),
        })
    }

    pub async fn default() -> TarsResult<Self> {
        TarsClient::new("http://127.0.0.1:42069".to_owned()).await
    }
}
