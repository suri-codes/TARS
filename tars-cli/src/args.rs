use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a task
    Add,

    /// List tasks
    List(ListArgs),
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(short, long)]
    /// The specific group youd like to see the tasks for
    group: Option<String>,
}
