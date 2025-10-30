use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::config::AppContext;

pub struct SearchService {
    root: PathBuf,
}

impl SearchService {
    pub fn new(ctx: &AppContext) -> Result<Self> {
        Ok(Self {
            root: ctx.vault_root().to_path_buf(),
        })
    }

    pub fn grep(&self, query: &str) -> Result<()> {
        let status = Command::new("rg")
            .arg("--hidden")
            .arg("--glob")
            .arg("!.git")
            .arg(query)
            .arg(".")
            .current_dir(&self.root)
            .status()
            .map_err(|err| map_exec_error(err, "rg"))?;
        if !status.success() {
            anyhow::bail!("ripgrep exited with {}", status);
        }
        Ok(())
    }

    pub fn fuzzy(&self, query: &str) -> Result<()> {
        let mut rg = Command::new("rg")
            .args(["--files"])
            .current_dir(&self.root)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| map_exec_error(err, "rg"))?;

        let mut fzf = Command::new("fzf")
            .arg("--ansi")
            .arg("--query")
            .arg(query)
            .stdin(
                rg.stdout
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("failed to capture ripgrep stdout"))?,
            )
            .spawn()
            .map_err(|err| map_exec_error(err, "fzf"))?;

        let status_fzf = fzf.wait()?;
        let status_rg = rg.wait()?;
        if !status_rg.success() {
            anyhow::bail!("ripgrep --files exited with {}", status_rg);
        }
        if !status_fzf.success() {
            anyhow::bail!("fzf exited with {}", status_fzf);
        }
        Ok(())
    }

    pub fn grep_matches(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let output = Command::new("rg")
            .arg("--hidden")
            .arg("--glob")
            .arg("!.git")
            .arg("--json")
            .arg(query)
            .arg(".")
            .current_dir(&self.root)
            .output()
            .map_err(|err| map_exec_error(err, "rg"))?;
        let success = output.status.success();
        let code = output.status.code();
        if !success && code != Some(1) {
            anyhow::bail!(
                "ripgrep exited with {}",
                output
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "unknown".into())
            );
        }
        let mut results = Vec::new();
        for line in output.stdout.split(|b| *b == b'\n') {
            if results.len() >= limit {
                break;
            }
            if line.is_empty() {
                continue;
            }
            let parsed: RgMessage = match serde_json::from_slice(line) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if let Some(data) = parsed.match_data() {
                let formatted = format!(
                    "{}:{}: {}",
                    data.path,
                    data.line_number,
                    data.line.trim_end_matches('\n')
                );
                results.push(formatted);
            }
        }
        Ok(results)
    }
}

#[derive(Debug, Deserialize)]
struct RgMessage {
    #[serde(rename = "type")]
    kind: String,
    data: Option<RgData>,
}

#[derive(Debug, Deserialize)]
struct RgData {
    path: RgText,
    lines: RgText,
    #[serde(rename = "line_number")]
    line_number: usize,
}

#[derive(Debug, Deserialize)]
struct RgText {
    text: String,
}

impl RgMessage {
    fn match_data(&self) -> Option<RgMatch> {
        if self.kind == "match" {
            self.data.as_ref().map(|data| RgMatch {
                path: data.path.text.clone(),
                line: data.lines.text.clone(),
                line_number: data.line_number,
            })
        } else {
            None
        }
    }
}

struct RgMatch {
    path: String,
    line: String,
    line_number: usize,
}

fn map_exec_error(err: io::Error, tool: &str) -> anyhow::Error {
    if err.kind() == io::ErrorKind::NotFound {
        anyhow!(
            r#"`{tool}` was not found in PATH. Please install it before using `obsctl search`. For example:
  • macOS (Homebrew): brew install {tool}
  • Ubuntu/Debian:   sudo apt-get install {tool}
  • Arch Linux:      sudo pacman -S {tool}"#
        )
    } else {
        anyhow::Error::new(err).context(format!("failed to execute {tool}"))
    }
}
