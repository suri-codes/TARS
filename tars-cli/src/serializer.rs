use std::fs;

use color_eyre::eyre::{Result, eyre};
use common::{
    TarsClient,
    types::{Color, Group, Task, TaskFetchOptions, parse_date_time},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::chrono::Local;

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

    fs::write(args.out_file.as_path(), export_info_str).map_err(|e| eyre!(e))?;

    println!("Exported to file {}", args.out_file.to_string_lossy());
    Ok(())
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

    let client = TarsClient::default().await?;

    for group in import_info.groups {
        group.raw_create(&client).await?;
    }

    for task in import_info.tasks {
        task.raw_create(&client).await?;
    }

    println!("Successfully imported tasks!");

    Ok(())
}

pub fn extract_task(task_json: &Value) -> Task {
    let id = task_json
        .get("id")
        .map(|i| {
            i.as_str()
                .expect("id should be a str at this point")
                .to_owned()
        })
        .unwrap_or_default();

    let name = task_json
        .get("name")
        .map(|n| {
            n.as_str()
                .expect("name should be a str at this point")
                .to_owned()
        })
        .unwrap_or_else(|| "Corrupted Name".to_owned());

    let group = extract_group(
        task_json
            .get("group")
            //NOTE: dont think there is a nice way to handle this
            .expect("Error while extracting! Task doesn't contain group!"),
    );

    let priority = task_json
        .get("priority")
        .map(|p| {
            p.as_str()
                .expect("priority should be a string")
                .try_into()
                .expect("priority should have been parsed properly")
        })
        .unwrap_or_default();

    let description = task_json
        .get("description")
        .map(|v| v.as_str().expect("description must be a string").to_owned())
        .unwrap_or_default();

    let finished_at = task_json.get("finished at").and_then(|v| {
        if let Some(str) = v.as_str()
            && v.is_string()
        {
            Some(parse_date_time(str).expect(
                "finish datetime exists but couldnt be
            parsed properly!",
            ))
        } else {
            None
        }
    });

    let created_at = if let Some(v) = task_json.get("created_at")
        && let Some(str) = v.as_str()
        && v.is_string()
    {
        parse_date_time(str).expect(
            "created_at datetime exists but couldnt be
            parsed properly!",
        )
    } else {
        Local::now().naive_local()
    };

    let due = task_json.get("due").and_then(|v| {
        if let Some(str) = v.as_str()
            && v.is_string()
        {
            Some(parse_date_time(str).expect(
                "due datetime exists but couldnt be
            parsed properly!",
            ))
        } else {
            None
        }
    });

    Task::with_all_fields(
        id,
        group,
        name,
        priority,
        description,
        finished_at,
        created_at,
        due,
    )
}

pub fn extract_group(group_json: &Value) -> Group {
    let id = group_json
        .get("id")
        .map(|i| {
            i.as_str()
                .expect("id should be a str at this point")
                .to_owned()
        })
        .unwrap_or_default();
    let name = group_json
        .get("name")
        .map(|n| {
            n.as_str()
                .expect("name should be a str at this point")
                .to_owned()
        })
        .unwrap_or_else(|| "Corrupted Name".to_owned());

    let parent_id = group_json.get("parent_id").map(|p_id| {
        p_id.as_str()
            .expect("parent_id should be a valid string")
            .to_owned()
            .into()
    });

    let priority = group_json
        .get("priority")
        .map(|p| {
            p.as_str()
                .expect("priority should be a string")
                .try_into()
                .expect("priority should have been parsed properly")
        })
        .unwrap_or_default();

    let color = group_json
        .get("color")
        .map(|c| {
            c.as_str()
                .expect("color should be a valid string")
                .to_owned()
                .try_into()
                .expect("the color should have been parsed properly")
        })
        .unwrap_or_else(Color::random);

    Group::with_all_fields(id, name, parent_id, priority, color)
}
