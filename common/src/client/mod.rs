use std::str::FromStr;

use reqwest::{Client, ClientBuilder, Url};
use tracing::error;

use crate::{
    TarsError,
    types::{Group, Task, TaskFetchOptions},
};

#[derive(Debug)]
pub struct TarsClient {
    pub conn: Client,
    pub base_path: Url,
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
            conn: client,
            base_path: Url::from_str(&base_url).expect("should be a valid url"),
        })
    }
    // / Returns ... from db of this [`TarsClient`].
    // /
    // /
    // / # Errors
    // /
    // / This function will return an error if .
    // pub async fn get_tasks(&mut self, opts: TaskFetchOptions) -> Result<Vec<Task>, TarsError> {
    //     let url = self.base_path.join("/task/fetch")?;

    //     let res: Vec<Task> = self
    //         .client
    //         .post(url)
    //         .json(&opts) // This is the correct way to send JSON with reqwest
    //         .send()
    //         .await
    //         .inspect_err(|e| error!("response {:?}", e))?
    //         .json()
    //         .await
    //         .inspect_err(|e| error!("json: {:?}", e))?;

    //     Ok(res)
    // }

    // pub async fn create_task(&mut self, task: Task) -> Result<Task, TarsError> {
    //     let url = self.base_path.join("/task/create")?;

    //     let res: Task = self
    //         .client
    //         .post(url)
    //         .json(&task)
    //         .send()
    //         .await
    //         .inspect_err(|e| error!("response {:?}", e))?
    //         .json()
    //         .await
    //         .inspect_err(|e| error!("json: {:?}", e))?;

    //     Ok(res)
    // }

    // pub async fn create_group(&mut self, group: Group) -> Result<Group, TarsError> {
    //     let url = self.base_path.join("/group/create")?;

    //     let res: Group = self
    //         .client
    //         .post(url)
    //         .json(&group)
    //         .send()
    //         .await
    //         .inspect_err(|e| error!("response {:?}", e))?
    //         .json()
    //         .await
    //         .inspect_err(|e| error!("json: {:?}", e))?;
    //     Ok(res)
    // }
}
