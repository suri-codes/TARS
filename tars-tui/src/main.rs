use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use common::logging;

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
    logging::init("tars-tui.log", true)?;

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    app.run().await?;
    Ok(())
}
