use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::owo_colors::OwoColorize;
use common::{
    ParseError,
    types::{Color, Id, Name, Priority},
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
    /// Manage tars groups.
    #[command(subcommand)]
    Group(GroupSubcommand),

    /// Manage tars tasks.
    #[command(subcommand)]
    Task(TaskSubcommand),
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
/// Arguments for adding a group.
pub struct GroupAddArgs {
    #[arg(short, long, value_parser=Name::parse_clap)]
    /// Name of group
    pub name: Name,

    #[arg(short, long, value_parser=Name::parse_clap)]
    /// Optional name of parent group.
    /// NOTE: Will be orphan if argument not provided or parent not found.
    pub parent: Option<Name>,

    #[arg(short, long,  value_parser=Color::parse_clap)]
    pub color: Option<Color>,
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
    #[arg(short = 'D', long, value_parser=parse_date_time)]
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

fn parse_date_time(arg: &str) -> Result<Option<NaiveDateTime>, ParseError> {
    match NaiveDateTime::parse_from_str(arg, "%Y-%m-%d %H:%M:%S") {
        Ok(parsed_time) => Ok(Some(parsed_time)),
        Err(_) => {
            println!("{}", "Failed to Parse, using None as due date".magenta());
            Err(ParseError::FailedToParse)
        }
    }
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
