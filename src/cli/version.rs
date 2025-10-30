use clap::Args;
use serde::Serialize;

#[derive(Args)]
pub struct VersionCommand {
    /// Output version information as JSON.
    #[arg(long)]
    pub json: bool,
    /// Show additional build metadata when available.
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Serialize)]
struct VersionInfo<'a> {
    name: &'a str,
    version: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_commit: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    build_timestamp: Option<&'a str>,
}

pub fn handle(cmd: VersionCommand) -> anyhow::Result<()> {
    let info = VersionInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        description: option_env!("CARGO_PKG_DESCRIPTION"),
        git_commit: option_env!("OBSCTL_GIT_COMMIT"),
        build_timestamp: option_env!("OBSCTL_BUILD_TS"),
    };

    if cmd.json {
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }

    println!("{} {}", info.name, info.version);
    if cmd.verbose {
        if let Some(desc) = info.description {
            println!("{desc}");
        }
        if let Some(commit) = info.git_commit {
            println!("commit: {commit}");
        }
        if let Some(ts) = info.build_timestamp {
            println!("built: {ts}");
        }
    }

    Ok(())
}
