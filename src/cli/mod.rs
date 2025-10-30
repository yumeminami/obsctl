mod config_cmd;
mod note;
mod search;
mod task;

use clap::{Parser, Subcommand};

/// CLI entry point for obsctl.
#[derive(Parser)]
#[command(name = "obsctl", version, about = "Local AI knowledge and task CLI")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    Note(note::NoteCommand),
    #[command(subcommand)]
    Task(task::TaskCommand),
    #[command(subcommand)]
    Search(search::SearchCommand),
    #[command(subcommand)]
    Config(config_cmd::ConfigCommand),
}

/// Parse CLI args and execute the command.
pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let ctx = crate::config::AppContext::load()?;

    match cli.command {
        Commands::Note(cmd) => note::handle(cmd, &ctx),
        Commands::Task(cmd) => task::handle(cmd, &ctx),
        Commands::Search(cmd) => search::handle(cmd, &ctx),
        Commands::Config(cmd) => config_cmd::handle(cmd, &ctx),
    }
}
