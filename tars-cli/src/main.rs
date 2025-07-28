use crate::args::{CliArgs, Commands};
use clap::Parser;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use common::TarsClient;
use handlers::{group_handler, task_handler};
use rustyline::{Config, Editor, history::FileHistory};
mod args;
mod handlers;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = CliArgs::parse();

    let client = TarsClient::default().await.unwrap();

    match args.command {
        Commands::Group(g_sub) => group_handler(&client, g_sub).await,

        Commands::Task(t_sub) => task_handler(&client, t_sub).await,
    }
}

#[allow(dead_code)]
fn prompt_user(prompt: &str) -> Result<String> {
    // look into rustlyline for saving things, might be super cool, or just not do that
    let mut rl: Editor<(), FileHistory> = Editor::with_config(
        Config::builder()
            .color_mode(rustyline::ColorMode::Enabled)
            .build(),
    )?;
    let colored_prompt = format!("{}: ", prompt.green());
    let response = rl.readline(&colored_prompt)?;
    Ok(response)
}
