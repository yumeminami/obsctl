use clap::{Args, Subcommand};

use crate::config::AppContext;
use crate::search::SearchService;

#[derive(Subcommand)]
pub enum SearchCommand {
    /// Run a fast literal/regex search using ripgrep.
    Grep(SearchArgs),
    /// Fuzzy-find note paths using fzf.
    Fzf(SearchArgs),
}

#[derive(Args)]
pub struct SearchArgs {
    /// Query string to search for.
    #[arg(required = true)]
    pub query: Vec<String>,
}

pub fn handle(cmd: SearchCommand, ctx: &AppContext) -> anyhow::Result<()> {
    let query = match &cmd {
        SearchCommand::Grep(args) | SearchCommand::Fzf(args) => args.query.join(" "),
    };
    let service = SearchService::new(ctx)?;
    match cmd {
        SearchCommand::Grep(_) => service.grep(&query)?,
        SearchCommand::Fzf(_) => service.fuzzy(&query)?,
    }
    Ok(())
}
