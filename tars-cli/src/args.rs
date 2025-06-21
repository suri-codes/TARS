use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::owo_colors::OwoColorize;
use common::{
    ParseError,
    types::{Name, Priority},
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
    /// Add a Group or Task.
    #[command(subcommand)]
    Add(AddType),

    /// List Groups or Tasks.
    List(ListArgs),
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(short, long)]
    /// The specific group youd like to see the tasks for
    group: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum AddType {
    Task(TaskAdd),
    Group(GroupAdd),
}

#[derive(Debug, Args)]
pub struct TaskAdd {
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
    #[arg(short, long, value_parser=parse_date_time)]
    pub due: Option<NaiveDateTime>,

    #[arg(short = 'D', long)]
    /// A description of the task at hand.
    pub description: String,
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

#[derive(Debug, Args)]
pub struct GroupAdd {
    #[arg(short, long, value_parser=Name::parse_clap)]
    pub name: String,

    #[arg(short, long, value_parser=Name::parse_clap)]
    pub parent: Option<String>,
}
