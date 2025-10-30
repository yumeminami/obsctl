mod config_cmd;
mod note;
mod search;
mod task;
mod version;

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
    /// Display version information.
    Version(version::VersionCommand),
}

/// Parse CLI args and execute the command.
pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version(cmd) => version::handle(cmd),
        Commands::Note(cmd) => {
            let ctx = crate::config::AppContext::load()?;
            note::handle(cmd, &ctx)
        }
        Commands::Task(cmd) => {
            let ctx = crate::config::AppContext::load()?;
            task::handle(cmd, &ctx)
        }
        Commands::Search(cmd) => {
            let ctx = crate::config::AppContext::load()?;
            search::handle(cmd, &ctx)
        }
        Commands::Config(cmd) => {
            let ctx = crate::config::AppContext::load()?;
            config_cmd::handle(cmd, &ctx)
        }
    }
}
