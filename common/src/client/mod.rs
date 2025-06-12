use std::str::FromStr;

use axum::Json;
use reqwest::{Client, ClientBuilder, Url};
use tracing::info;

use crate::{
    TarsError,
    types::{Task, TaskFetchOptions},
};

#[derive(Debug)]
pub struct TarsClient {
    client: Client,
    base_path: Url,
}

// fix the docs for these
impl TarsClient {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub async fn new(base_url: String) -> Result<Self, TarsError> {
        let app_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let client = ClientBuilder::new().user_agent(app_agent).build()?;

        Ok(Self {
            client,
            base_path: Url::from_str(&base_url).expect("should be a valid url"),
        })
    }
    /// Returns ... from db of this [`TarsClient`].
    ///
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    #[expect(unused)]
    pub async fn get_tasks(&mut self, opts: TaskFetchOptions) -> Result<Vec<Task>, TarsError> {
        let url = self.base_path.join("/task/fetch")?;

        println!("{:?}", url);

        // let res: Vec<Task> = self
        //     .client
        //     .post(url)
        //     .json(&opts) // This is the correct way to send JSON with reqwest
        //     .send()
        //     .await
        //     .inspect_err(|e| println!("response {:?}", e))?
        //     .json()
        //     .await
        //     .inspect_err(|e| println!("json {:?}", e))?;

        let res = self
            .client
            .post(self.base_path.join("/task/test")?)
            .send()
            .await
            .inspect_err(|e| println!("response {:?}", e))?;

        println!("{:?}", res);

        let res = self
            .client
            .post(url)
            .json(&opts) // This is the correct way to send JSON with reqwest
            .send()
            .await
            .inspect_err(|e| println!("response {:?}", e))?;

        println!("{:?}", res);

        todo!()
        // Ok(res)
    }
}
