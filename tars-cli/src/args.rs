use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::owo_colors::OwoColorize;
use common::{
    ParseError,
    types::{Color, Id, Name, Priority, parse_date_time},
};
use sqlx::types::chrono::NaiveDateTime;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage TARS groups.
    #[command(subcommand)]
    Group(GroupSubcommand),

    /// Manage TARS tasks.
    #[command(subcommand)]
    Task(TaskSubcommand),

    /// Exports TARS data.
    Export(ExportArgs),

    /// Imports bulk data into TARS
    /// NOTE: By default the importer will fill in fields with
    // default values if they arent present / aren't able to be
    // parsed properly
    Import(ImportArgs),
}

#[derive(Subcommand, Debug)]
/// Subcommand to manage tars groups.
pub enum GroupSubcommand {
    /// Add a group.
    Add(GroupAddArgs),
    /// List groups.
    List(GroupListArgs),
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(short, long, default_value = "./tars.json")]
            let (s_key, v_key) = SigningKey::new_keypair(&SigningScheme::Ecdsa, Default::default())
                .expect("should have worked");
    /// The file-path for data to pe put into.
    pub out_file: PathBuf,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[arg(short, long)]
    /// The file-path for data to sourced from.
    pub in_file: PathBuf,

    #[arg(short, long, default_value = "false")]
    /// Will make the importer import strictly, failing on any schema mismatch
    /// or missing fields.
    pub strict: bool,
}

#[derive(Debug, Args)]
/// Arguments for adding a group.
pub struct GroupAddArgs {
    #[arg(short, long, value_parser=Name::parse_clap)]
    /// Name of group
    pub name: Name,

    #[arg(short, long, value_parser=Name::parse_clap)]
    /// Optional name of parent group.
    /// NOTE: Will be orphan if argument not provided or parent not found.
    pub parent: Option<Name>,

    #[arg(short, long,  value_parser=Color::parse_str)]
    pub color: Option<Color>,

    #[arg(short, long, value_parser=Priority::parse_clap)]
    /// The priority of the group
    pub priority: Option<Priority>,
}

#[derive(Debug, Args)]
/// Arguments for listing groups.
pub struct GroupListArgs {
    #[arg(short, long, value_parser=Id::parse_clap)]
    pub id: Option<Id>,
    #[arg(short, long, value_parser=Name::parse_clap)]
    pub name: Option<Name>,
}

#[derive(Subcommand, Debug)]
/// Subcommand to mange tars tasks.
pub enum TaskSubcommand {
    /// Add a task.
    Add(TaskAddArgs),
    /// List tasks.
    List(TaskListArgs),
}

#[derive(Debug, Args)]
/// Arguments for adding a task.
pub struct TaskAddArgs {
    #[arg(short, long, value_parser=Name::parse_clap)]
    /// The name of the task.
    pub name: Name,

    #[arg(short, long, value_parser=Name::parse_clap)]
    /// The group the task should belong too.
    pub group: Name,

    #[arg(short, long)]
    /// The priority of the task.
    pub priority: PriorityInput,

    /// An optional Due Date.
    #[arg(short = 'D', long, value_parser=clap_parse_date)]
    pub due: Option<NaiveDateTime>,

    #[arg(short, long)]
    /// A description of the task at hand.
    pub description: String,
}

#[derive(Debug, Args)]
/// Arguments for listing tasks.
pub struct TaskListArgs {
    #[arg(short, long)]
    /// The specific group youd like to see the tasks for
    #[arg(short='n', long, value_parser=Name::parse_clap)]
    group_name: Option<Name>,

    #[arg(short='i', long, value_parser=Id::parse_clap)]
    group_id: Option<Id>,

    #[arg(short, long)]
    unfinished: Option<bool>,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum PriorityInput {
    Low,
    Medium,
    High,
    Asap,
    Far,
}

impl From<PriorityInput> for Priority {
    fn from(value: PriorityInput) -> Self {
        match value {
            PriorityInput::Low => Priority::Low,
            PriorityInput::Medium => Priority::Medium,
            PriorityInput::High => Priority::High,
            PriorityInput::Asap => Priority::Asap,
            PriorityInput::Far => Priority::Far,
        }
    }
}

pub fn clap_parse_date(arg: &str) -> Result<Option<NaiveDateTime>, ParseError> {
    if arg.is_empty() {
        println!("{}", "No date provided!, using None as due date".yellow());
        return Ok(None);
    }

    match parse_date_time(arg) {
        Ok(res) => Ok(Some(res)),
        Err(e) => {
            let str = e.to_string();
            println!("{}", str.magenta());
            Err(e)
        }
    }
}
