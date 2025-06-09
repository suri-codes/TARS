use reqwest::{Client, ClientBuilder, Error};

use crate::{TarsError, types::Task};

#[derive(Debug)]
struct TarsClient {
    client: Client,
    base_url: String,
}

impl TarsClient {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    async fn new(base_url: String) -> Result<Self, TarsError> {
        let app_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let client = ClientBuilder::new().user_agent(app_agent).build()?;
        todo!()
    }
    /// Returns ... from db of this [`TarsClient`].
    ///
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    async fn get_tasks(&mut self) -> Result<Vec<Task>, TarsError> {
        let resp: Vec<Task> = self
            .client
            .get(self.base_url.clone())
            .send()
            .await?
            .json()
            .await?;

        todo!()
    }
}
