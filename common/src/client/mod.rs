use std::str::FromStr;

use reqwest::{Client, ClientBuilder, Url};

use crate::TarsError;
/// Holds the reqwest `Client` and the base path for accessing the `TarsDaemon`
#[derive(Debug)]
pub struct TarsClient {
    pub base_path: Url,
    pub conn: Client,
}

impl TarsClient {
    /// Creates a new TarsClient with the provided base_url.
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    /// + Connecting to the daemon fails.
    pub async fn new(base_url: String) -> Result<Self, TarsError> {
        let app_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let client = ClientBuilder::new().user_agent(app_agent).build()?;

        Ok(Self {
            conn: client,
            base_path: Url::from_str(&base_url).expect("should be a valid url"),
        })
    }
}
