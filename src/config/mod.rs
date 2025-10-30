use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::templates;

const ROOT_DIR_NAME: &str = ".obsctl";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone)]
pub struct AppContext {
    config_path: PathBuf,
    config: AppConfig,
    vault_root: PathBuf,
}

impl AppContext {
    pub fn load() -> Result<Self> {
        let config_path = default_config_path()?;
        let manager = ConfigManager::new(config_path.clone());
        if !config_path.exists() {
            manager.ensure_initialized(None)?;
        }
        let config = manager.load()?;
        let vault_root = PathBuf::from(&config.vault.path);
        if !vault_root.exists() {
            manager.ensure_initialized(Some(&vault_root))?;
        }
        Ok(Self {
            config_path,
            vault_root,
            config,
        })
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn config_file(&self) -> &Path {
        &self.config_path
    }

    pub fn vault_root(&self) -> &Path {
        &self.vault_root
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub vault: VaultConfig,
    pub templates: TemplateConfig,
    pub search: SearchConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultConfig {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateConfig {
    pub daily: String,
    pub task: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
    pub tool: String,
    pub fzf_preview: bool,
}

impl AppConfig {
    fn new(vault_root: &Path) -> Self {
        let vault_str = vault_root.to_string_lossy().to_string();
        let templates_dir = vault_root.join("templates");
        AppConfig {
            vault: VaultConfig {
                path: vault_str.clone(),
            },
            templates: TemplateConfig {
                daily: templates_dir.join("daily.md").to_string_lossy().to_string(),
                task: templates_dir.join("task.md").to_string_lossy().to_string(),
            },
            search: SearchConfig {
                tool: "ripgrep".to_string(),
                fzf_preview: true,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    path: PathBuf,
}

impl ConfigManager {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<AppConfig> {
        let text = fs::read_to_string(&self.path)
            .with_context(|| format!("read config file {}", self.path.display()))?;
        let cfg: AppConfig = toml::from_str(&text).with_context(|| "parse obsctl configuration")?;
        Ok(cfg)
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        let text = toml::to_string_pretty(config)?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, text)?;
        Ok(())
    }

    pub fn ensure_initialized(&self, explicit_vault: Option<&Path>) -> Result<()> {
        let vault_root = explicit_vault
            .map(Path::to_path_buf)
            .unwrap_or(default_vault_path()?);

        let mut config = if self.path.exists() {
            let mut existing = self.load()?;
            if explicit_vault.is_some() {
                existing.vault.path = vault_root.to_string_lossy().to_string();
                self.save(&existing)?;
            }
            existing
        } else {
            let config = AppConfig::new(&vault_root);
            self.save(&config)?;
            config
        };

        if explicit_vault.is_some() {
            config.vault.path = vault_root.to_string_lossy().to_string();
            self.save(&config)?;
        }

        self.ensure_directories(&vault_root)?;
        templates::install_defaults(&vault_root)?;
        Ok(())
    }

    pub fn update_vault_path(&self, new_path: &Path) -> Result<()> {
        let mut cfg = self.load()?;
        cfg.vault.path = new_path.to_string_lossy().to_string();
        self.save(&cfg)?;
        self.ensure_directories(new_path)?;
        templates::install_defaults(new_path)?;
        Ok(())
    }

    fn ensure_directories(&self, vault_root: &Path) -> Result<()> {
        fs::create_dir_all(vault_root)?;
        for dir in ["Journal", "Tasks", "Projects", "templates"] {
            fs::create_dir_all(vault_root.join(dir))?;
        }
        let tasks_file = vault_root.join("Tasks/tasks.md");
        if !tasks_file.exists() {
            fs::write(&tasks_file, "# Tasks\n\n")?;
        }
        Ok(())
    }
}

fn default_root_dir() -> Result<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home).join(ROOT_DIR_NAME));
    }
    if let Ok(profile) = env::var("USERPROFILE") {
        return Ok(PathBuf::from(profile).join(ROOT_DIR_NAME));
    }
    anyhow::bail!("unable to locate home directory for obsctl config");
}

fn default_config_path() -> Result<PathBuf> {
    Ok(default_root_dir()?.join(CONFIG_FILE_NAME))
}

fn default_vault_path() -> Result<PathBuf> {
    Ok(default_root_dir()?.join("vault"))
}
