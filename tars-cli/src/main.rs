use crate::args::{CliArgs, Commands};
use clap::Parser;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use common::{
    TarsClient,
    types::{Group, Priority, Task, TaskFetchOptions},
};
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

    // match args.command {
    //     Commands::Add(AddType::Task(t)) => {
    //         let all = Group::fetch_all(&client).await?;
    //         let existing = all.iter().find(|e| **e.name == *t.group);

    //         let g = match existing {
    //             Some(o) => o.clone(),
    //             None => Group::new(&client, t.group, None).await?,
    //         };

    //         let task =
    //             Task::new(&client, &g, t.name, t.priority.into(), t.description, t.due).await?;

    //         println!("Added Task: {:#?}", task);
    //     }
    //     Commands::Add(AddType::Group(g)) => {
    //         let parent_id = if let Some(parent_name) = g.parent {
    //             let all = Group::fetch_all(&client).await?;
    //             all.iter().find_map(|g| {
    //                 if *g.name == parent_name {
    //                     Some(g.id.clone())
    //                 } else {
    //                     None
    //                 }
    //             })
    //         } else {
    //             None
    //         };

    //         let g = Group::new(&client, g.name, parent_id).await?;
    //         println!("Added Group: {:#?}", g);
    //     }

    //     Commands::List(_l_args) => {
    //         let group = Group::new(&client, "Penis", None).await?;
    //         let group_2 = Group::new(&client, "lol", Some(group.id)).await?;

    //         let _task = Task::new(
    //             &client,
    //             &group_2,
    //             "Kill Albert",
    //             Priority::Asap,
    //             "Albert playing too much league, its time to js kill bro",
    //             None,
    //         )
    //         .await?;

    //         let tasks = Task::fetch(&client, TaskFetchOptions::All).await?;

    //         println!("{:?}", tasks);
    //     }
    // }
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
    // print!("{}: ", prompt.green());
    // stdout().flush()?;
    // let mut input = String::new();
    // stdin().read_line(&mut input)?;

    // let input = input.trim();
    // Ok(input.to_owned())
}
