use std::io::{Write, stdin, stdout};

use crate::args::{CliArgs, Commands};
use clap::Parser;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use common::types::{Group, Name, Priority, Task};
use sqlx::types::chrono::NaiveDateTime;
mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = CliArgs::parse();

    match args.command {
        Commands::Add => {
            let name: Name = prompt_user("Task Name")?.as_str().try_into()?;

            //TODO: print existing groups
            let group: Group = prompt_user("Group Name")?.as_str().try_into()?;
            let priority: Priority =
                prompt_user("Priority Level [(L)ow|(M)edium|(H)igh|(A)SAP|(F)ar]")?
                    .as_str()
                    .try_into()?;

            let description = prompt_user("Task Description")?;

            let due_str = prompt_user("Due Date (YYYY-MM-DD HH:MM:SS)")?;

            let due = match NaiveDateTime::parse_from_str(&due_str, "%Y-%m-%d %H:%M:%S") {
                Ok(parsed_time) => Some(parsed_time),
                Err(_) => {
                    println!("{}", "Failed to Parse, using None as due date".magenta());
                    None
                }
            };

            let _task = Task::new(group, name, priority, description, due);
            // orm.insert_task(task).await?;
        }
        Commands::List(_l_args) => {
            // need to list all the groups
            // let tasks = orm.fetch_tasks(FetchOptions{ fetch_type: todo!(), completion_status: todo!() });
            todo!()
        }
    }

    Ok(())
}

fn prompt_user(prompt: &str) -> Result<String> {
    //TODO: priority not implemented correctly
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
