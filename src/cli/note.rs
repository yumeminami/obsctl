use clap::{Args, Subcommand};

use crate::config::AppContext;
use crate::core::vault::VaultService;

#[derive(Subcommand)]
pub enum NoteCommand {
    /// Append a line to today's daily note.
    Add(NoteAddArgs),
    /// Print the path to today's (or a specific date's) note.
    Open(NoteOpenArgs),
    /// List the most recent daily notes.
    List(NoteListArgs),
}

#[derive(Args)]
pub struct NoteAddArgs {
    /// Content to append into the daily note.
    #[arg(required = true)]
    pub entry: Vec<String>,
}

#[derive(Args)]
pub struct NoteOpenArgs {
    /// Optional ISO date (YYYY-MM-DD) to open.
    #[arg(long)]
    pub date: Option<String>,
}

#[derive(Args)]
pub struct NoteListArgs {
    /// Number of recent notes to list.
    #[arg(long, default_value_t = 5)]
    pub limit: usize,
}

pub fn handle(cmd: NoteCommand, ctx: &AppContext) -> anyhow::Result<()> {
    let service = VaultService::new(ctx)?;
    match cmd {
        NoteCommand::Add(args) => {
            let text = args.entry.join(" ");
            service.append_today(&text)?;
            println!("Appended to {}", service.today_path().display());
        }
        NoteCommand::Open(args) => {
            let path = service.path_for(args.date.as_deref())?;
            println!("{}", path.display());
        }
        NoteCommand::List(args) => {
            for path in service.list_recent(args.limit)? {
                println!("{}", path.display());
            }
        }
    }
    Ok(())
}
