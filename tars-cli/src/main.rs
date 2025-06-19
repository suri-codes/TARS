use std::io::{Write, stdin, stdout};

use crate::args::{CliArgs, Commands};
use clap::Parser;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use common::{
    TarsClient,
    types::{Group, Name, Priority, Task, TaskFetchOptions},
};
use sqlx::types::chrono::NaiveDateTime;
mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = CliArgs::parse();

    match args.command {
        Commands::Add => {
            let _name: Name = prompt_user("Task Name")?.as_str().into();

            //TODO: print existing groups
            // let group: Group = prompt_user("Group Name")?.as_str().try_into()?;
            let _priority: Priority =
                prompt_user("Priority Level [(L)ow|(M)edium|(H)igh|(A)SAP|(F)ar]")?
                    .as_str()
                    .try_into()?;

            let _description = prompt_user("Task Description")?;

            let due_str = prompt_user("Due Date (YYYY-MM-DD HH:MM:SS)")?;

            let _due = match NaiveDateTime::parse_from_str(&due_str, "%Y-%m-%d %H:%M:%S") {
                Ok(parsed_time) => Some(parsed_time),
                Err(_) => {
                    println!("{}", "Failed to Parse, using None as due date".magenta());
                    None
                }
            };
        }
        Commands::List(_l_args) => {
            let client = TarsClient::new("http://127.0.0.1:42069".to_owned())
                .await
                .unwrap();

            let group = Group::new(&client, "Penis", None).await?;
            let group_2 = Group::new(&client, "lol", Some(group.id)).await?;

            let _task = Task::new(
                &client,
                &group_2,
                "Kill Albert",
                Priority::Asap,
                "Albert playing too much league, its time to js kill bro",
                None,
            )
            .await?;

            let tasks = Task::fetch(&client, TaskFetchOptions::All).await?;

            println!("{:?}", tasks);
        }
    }

    Ok(())
}

fn prompt_user(prompt: &str) -> Result<String> {
    // look into rustlyline for saving things, might be super cool, or just not do that
    // let mut rl = Editor::with_config(
    //     Config::builder()
    //         .color_mode(rustyline::ColorMode::Enabled)
    //         .build(),
    // )?;
    // let colored_prompt = format!("{}: ", prompt);
    // let response = rl.readline(colored_prompt)?;
    // Ok(response)
    print!("{}: ", prompt.green());
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    let input = input.trim();
    Ok(input.to_owned())
}
