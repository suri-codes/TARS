use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use common::{TarsClient, logging};

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    logging::init("tars-tui.log", false)?;

    let args = Cli::parse();

    let client = TarsClient::new("http://127.0.0.1:42069".to_owned())
        .await
        .unwrap();

    let mut app = App::new(client, args.tick_rate, args.frame_rate)?;
    app.run().await?;
    Ok(())
}
