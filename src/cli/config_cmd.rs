use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::config::{AppContext, ConfigManager};

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Initialize the vault layout and default config.
    Init(ConfigInitArgs),
    /// Show or update the configured vault path.
    Path(ConfigPathArgs),
}

#[derive(Args)]
pub struct ConfigInitArgs {
    /// Optional explicit vault directory path.
    #[arg(long)]
    pub vault: Option<PathBuf>,
}

#[derive(Args)]
pub struct ConfigPathArgs {
    /// Update the vault path to the provided location.
    #[arg(long)]
    pub set: Option<PathBuf>,
}

pub fn handle(cmd: ConfigCommand, ctx: &AppContext) -> anyhow::Result<()> {
    match cmd {
        ConfigCommand::Init(args) => {
            let manager = ConfigManager::new(ctx.config_file().to_path_buf());
            manager.ensure_initialized(args.vault.as_deref())?;
            println!(
                "Vault initialized at {}",
                PathBuf::from(manager.load()?.vault.path).display()
            );
        }
        ConfigCommand::Path(args) => {
            let manager = ConfigManager::new(ctx.config_file().to_path_buf());
            if let Some(path) = args.set {
                manager.update_vault_path(&path)?;
                println!("Updated vault path to {}", path.display());
            } else {
                println!("{}", ctx.vault_root().display());
            }
        }
    }
    Ok(())
}
