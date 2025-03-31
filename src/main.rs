use args::{CliArgs, Commands};
use clap::Parser;
use color_eyre::eyre::Result;
use db::Db;

mod args;
mod db;
mod dirs;
#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    let db = Db::connect().await?;
    // println!("Hello, world!");
    match args.command {
        Commands::Add => {}
        Commands::List(l_args) => {}
    }

    Ok(())
}
