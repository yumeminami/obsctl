use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};

use crate::config::AppContext;
use crate::templates;

pub struct VaultService {
    root: PathBuf,
    journal_dir: PathBuf,
}

impl VaultService {
    pub fn new(ctx: &AppContext) -> Result<Self> {
        let root = ctx.vault_root().to_path_buf();
        let journal_dir = root.join("Journal");
        fs::create_dir_all(&journal_dir)?;
        Ok(Self { root, journal_dir })
    }

    pub fn append_today(&self, text: &str) -> Result<()> {
        let date = Local::now().date_naive();
        self.append_for_date(date, text)
    }

    pub fn today_path(&self) -> PathBuf {
        self.journal_dir
            .join(Self::filename_for(Local::now().date_naive()))
    }

    pub fn path_for(&self, date: Option<&str>) -> Result<PathBuf> {
        let target = match date {
            Some(text) => NaiveDate::parse_from_str(text, "%Y-%m-%d")
                .with_context(|| format!("invalid date format: {text}"))?,
            None => Local::now().date_naive(),
        };
        let path = self.journal_dir.join(Self::filename_for(target));
        self.ensure_daily_file(&path, target)?;
        Ok(path)
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<PathBuf>> {
        let mut entries: Vec<_> = fs::read_dir(&self.journal_dir)?
            .filter_map(|res| res.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file() && path.extension().map(|ext| ext == "md").unwrap_or(false)
            })
            .collect();
        entries.sort_by(|a, b| b.cmp(a));
        entries.truncate(limit);
        Ok(entries)
    }

    fn append_for_date(&self, date: NaiveDate, text: &str) -> Result<()> {
        let path = self.journal_dir.join(Self::filename_for(date));
        self.ensure_daily_file(&path, date)?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .with_context(|| format!("open daily note {}", path.display()))?;
        writeln!(file, "{}", text)?;
        Ok(())
    }

    fn ensure_daily_file(&self, path: &Path, date: NaiveDate) -> Result<()> {
        if path.exists() {
            return Ok(());
        }

        let template = templates::load_daily_template(&self.root)?;
        let filled = template.replace("{{date}}", &date.format("%Y-%m-%d").to_string());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, filled)?;
        Ok(())
    }

    fn filename_for(date: NaiveDate) -> String {
        format!("{}.md", date.format("%Y-%m-%d"))
    }
}
