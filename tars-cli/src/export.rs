use std::fs;

use color_eyre::eyre::{Result, eyre};
use common::{
    TarsClient,
    types::{Color, Group, Task, TaskFetchOptions},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::args::{ExportArgs, ImportArgs};

#[derive(Serialize, Deserialize)]
struct SerializedInfo {
    groups: Vec<Group>,
    tasks: Vec<Task>,
}

pub async fn export(args: ExportArgs) -> Result<()> {
    let client = TarsClient::default().await?;
    let tasks = Task::fetch(&client, TaskFetchOptions::All).await?;
    let groups = Group::fetch_all(&client).await?;

    let export_info = SerializedInfo { groups, tasks };

    let export_info_str = serde_json::to_string_pretty(&export_info)?;

    fs::write(args.out_file, export_info_str).map_err(|e| eyre!(e))
}

pub async fn import(args: ImportArgs) -> Result<()> {
    let import_info_str = fs::read_to_string(args.in_file)?;

    let import_info: SerializedInfo = if args.strict {
        serde_json::from_str(&import_info_str)?
    } else {
        let intermediate: Value = serde_json::from_str(&import_info_str)?;

        // first we get the groups
        let groups_arr = intermediate
            .get("groups")
            .ok_or(eyre!("should have a group array in the base json object!"))?;

        let tasks_arr = intermediate
            .get("tasks")
            .ok_or(eyre!("should have a task array in the base json object!"))?;

        let mut group_i = 0;

        let mut groups = Vec::new();

        while let Some(group_json) = groups_arr.get(group_i) {
            let group = extract_group(group_json);
            groups.push(group);
            group_i += 1;
        }

        let mut task_i = 0;
        let mut tasks = Vec::new();

        while let Some(task_json) = tasks_arr.get(task_i) {
            let task = extract_task(task_json);

            tasks.push(task);
            task_i += 1;
        }

        SerializedInfo { groups, tasks }
    };

    Ok(())
}

pub fn extract_task(task_json: &Value) -> Task {
    todo!()
}

pub fn extract_group(group_json: &Value) -> Group {
    Group::with_all_fields(
        group_json
            .get("id")
            .map(|i| {
                i.as_str()
                    .expect("id should be a str at this point")
                    .to_owned()
            })
            .unwrap_or_default(),
        group_json
            .get("name")
            .map(|n| {
                n.as_str()
                    .expect("name should be a str at this point")
                    .to_owned()
            })
            .unwrap_or_else(|| "Corrupted Name".to_owned()),
        group_json.get("parent_id").map(|p_id| {
            p_id.as_str()
                .expect("parent_id should be a valid string")
                .to_owned()
                .into()
        }),
        group_json
            .get("priority")
            .map(|p| {
                p.as_str()
                    .expect("priority should be a string")
                    .try_into()
                    .expect("priority should have been parsed properly")
            })
            .unwrap_or_default(),
        group_json
            .get("color")
            .map(|c| {
                c.as_str()
                    .expect("color should be a valid string")
                    .to_owned()
                    .try_into()
                    .expect("the color should have been parsed properly")
            })
            .unwrap_or_else(Color::random),
    )
}
