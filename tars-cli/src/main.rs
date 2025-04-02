use crate::args::{CliArgs, Commands};
use clap::Parser;
use color_eyre::eyre::Result;
use common::db::Db;
mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    let db = Db::connect().await?;

    match args.command {
        Commands::Add => {
            // need to add to the thang
        }
        Commands::List(l_args) => {
            // need to list all the groups
        }
    }

    Ok(())
}
