use color_eyre::eyre::Result;
use common::{
    TarsClient,
    types::{Color, Group, Task, TaskFetchOptions},
};

use crate::args::{GroupSubcommand, TaskSubcommand};

pub async fn task_handler(client: &TarsClient, t_sub: TaskSubcommand) -> Result<()> {
    match t_sub {
        TaskSubcommand::Add(args) => {
            let all = Group::fetch_all(client).await?;
            let existing = all.iter().find(|e| e.name == args.group);

            let g = match existing {
                Some(g) => g.to_owned(),
                None => {
                    let g = Group::new(client, args.group, None, Color::default()).await?;
                    eprintln!("Created new group: {g}");
                    g
                }
            };

            let task = Task::new(
                client,
                &g,
                args.name,
                args.priority.into(),
                args.description,
                args.due,
            )
            .await?;

            println!("Added Task:\n{task}");
        }
        TaskSubcommand::List(_args) => {
            // TODO: we need to filter on the arguments / debate whether we
            // want to do this filtering on the serverf side through taskfetchoptions.
            let all_tasks = Task::fetch(client, TaskFetchOptions::All).await?;

            for t in all_tasks.iter() {
                println!("{t}");
                println!("====================================================")
            }
        }
    }
    Ok(())
}
pub async fn group_handler(client: &TarsClient, g_sub: GroupSubcommand) -> Result<()> {
    match g_sub {
        GroupSubcommand::Add(args) => {
            let parent_id = if let Some(parent_name) = args.parent {
                let all = Group::fetch_all(client).await?;
                all.iter().find_map(|g| {
                    if *g.name == *parent_name {
                        Some(g.id.clone())
                    } else {
                        None
                    }
                })
            } else {
                None
            };

            let g =
                Group::new(client, args.name, parent_id, args.color.unwrap_or_default()).await?;
            println!("Added Group: {g}");
        }
        GroupSubcommand::List(args) => {
            let groups = Group::fetch_all(client).await?;

            let filtered: Vec<Group> = groups
                .into_iter()
                .filter(|e| {
                    // fall into here if provided some id
                    if let Some(target_id) = &args.id {
                        return *target_id == e.id;
                    }

                    // fall into here if provided some name
                    if let Some(target_name) = &args.name {
                        return *target_name == e.name;
                    }

                    // default case
                    true
                })
                .collect();

            for g in filtered {
                println!("{g}");
                println!("====================================================")
            }
        }
    }
    Ok(())
}
