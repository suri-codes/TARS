use color_eyre::eyre::Result;
use common::{
    TarsClient,
    types::{Group, Task},
};

use crate::args::{GroupSubcommand, TaskSubcommand};

pub async fn task_handler(client: &TarsClient, t_sub: TaskSubcommand) -> Result<()> {
    match t_sub {
        TaskSubcommand::Add(args) => {
            let all = Group::fetch_all(client).await?;
            let existing = all.iter().find(|e| e.name == args.group);

            let g = match existing {
                Some(g) => g.to_owned(),
                None => Group::new(client, args.group, None).await?,
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

            println!("Added Task: {:#?}", task);
        }
        TaskSubcommand::List(_args) => {
            todo!();
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

            let g = Group::new(client, args.name, parent_id).await?;
            println!("Added Group: {:#?}", g);
        }
        GroupSubcommand::List(_args) => {}
    }
    Ok(())
}
