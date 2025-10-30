use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

const DAILY_TEMPLATE: &str = r#"# Daily {{date}}

## Highlights

- 

## Tasks

- [ ] 

## Notes

- 
"#;

const TASK_TEMPLATE: &str = r#"# Tasks

- [ ] Example task
"#;

pub fn install_defaults(vault_root: &Path) -> Result<()> {
    let templates_dir = vault_root.join("templates");
    fs::create_dir_all(&templates_dir)?;
    ensure_file(&templates_dir.join("daily.md"), DAILY_TEMPLATE)?;
    ensure_file(&templates_dir.join("task.md"), TASK_TEMPLATE)?;
    Ok(())
}

pub fn load_daily_template(vault_root: &Path) -> Result<String> {
    let path = vault_root.join("templates/daily.md");
    load_template(&path, DAILY_TEMPLATE)
}

pub fn load_task_template(vault_root: &Path) -> Result<String> {
    let path = vault_root.join("templates/task.md");
    load_template(&path, TASK_TEMPLATE)
}

fn load_template(path: &Path, default: &str) -> Result<String> {
    ensure_file(path, default)?;
    fs::read_to_string(path).with_context(|| format!("read template {}", path.display()))
}

fn ensure_file(path: &Path, default: &str) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, default)?;
    }
    Ok(())
}
