use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::config::AppContext;
use crate::templates;

pub struct TaskService {
    tasks_file: PathBuf,
}

impl TaskService {
    pub fn new(ctx: &AppContext) -> Result<Self> {
        let vault_root = ctx.vault_root().to_path_buf();
        let tasks_file = vault_root.join("Tasks/tasks.md");
        if !tasks_file.exists() {
            if let Some(parent) = tasks_file.parent() {
                fs::create_dir_all(parent)?;
            }
            let template = templates::load_task_template(&vault_root)?;
            fs::write(&tasks_file, template)?;
        }
        Ok(Self { tasks_file })
    }

    pub fn add_task(&self, new_task: NewTask) -> Result<usize> {
        let records = self.read_records()?;
        let next_id = records.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        let rendered = new_task.render(next_id);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.tasks_file)
            .with_context(|| format!("open tasks file {}", self.tasks_file.display()))?;
        if !self.file_ends_with_newline()? {
            writeln!(file)?;
        }
        writeln!(file, "{rendered}")?;
        Ok(next_id)
    }

    pub fn mark_done(&self, id: usize) -> Result<()> {
        self.set_status(id, true)
    }

    pub fn set_status(&self, id: usize, done: bool) -> Result<()> {
        let mut updated = false;
        let lines: Vec<String> = self
            .read_lines()?
            .into_iter()
            .map(|line| {
                if let Some(record) = TaskRecord::parse(&line) {
                    if record.id == id {
                        updated = true;
                        return record.into_status_line(done);
                    }
                }
                line
            })
            .collect();
        if !updated {
            return Err(anyhow!("task #{id} was not found"));
        }
        self.write_lines(lines)
    }

    pub fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<String>> {
        let records = self.read_records()?;
        let items = records
            .into_iter()
            .filter(|r| match filter {
                TaskFilter::All => true,
                TaskFilter::Open => !r.done,
                TaskFilter::Done => r.done,
            })
            .map(|r| r.raw_text)
            .collect();
        Ok(items)
    }

    pub fn clean_completed(&self) -> Result<()> {
        let lines: Vec<String> = self
            .read_lines()?
            .into_iter()
            .filter(|line| TaskRecord::parse(line).map(|r| !r.done).unwrap_or(true))
            .collect();
        self.write_lines(lines)
    }

    pub fn find_task_by_title(&self, title: &str) -> Result<Option<TaskEntry>> {
        let title_lower = title.to_lowercase();
        for record in self.read_records()? {
            if record.title.to_lowercase() == title_lower {
                return Ok(Some(record.into_entry()));
            }
        }
        Ok(None)
    }

    pub fn tasks(&self) -> Result<Vec<TaskEntry>> {
        self.read_records()
            .map(|records| records.into_iter().map(TaskRecord::into_entry).collect())
    }

    fn read_lines(&self) -> Result<Vec<String>> {
        let content = fs::read_to_string(&self.tasks_file)
            .with_context(|| format!("read tasks file {}", self.tasks_file.display()))?;
        Ok(content.lines().map(|line| line.to_string()).collect())
    }

    fn read_records(&self) -> Result<Vec<TaskRecord>> {
        Ok(self
            .read_lines()?
            .into_iter()
            .filter_map(|line| TaskRecord::parse(&line))
            .collect())
    }

    fn write_lines(&self, lines: Vec<String>) -> Result<()> {
        let text = if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n") + "\n"
        };
        fs::write(&self.tasks_file, text)?;
        Ok(())
    }

    fn file_ends_with_newline(&self) -> Result<bool> {
        if !self.tasks_file.exists() {
            return Ok(true);
        }
        let metadata = fs::metadata(&self.tasks_file)?;
        if metadata.len() == 0 {
            return Ok(true);
        }
        let content = fs::read(&self.tasks_file)?;
        Ok(content.last().map(|b| *b == b'\n').unwrap_or(false))
    }
}

pub struct NewTask {
    pub title: String,
    pub due_date: Option<String>,
    pub recurrence: Option<String>,
    pub priority: Option<Priority>,
}

impl NewTask {
    fn render(&self, id: usize) -> String {
        let mut line = format!("- [ ] ({id}) {}", self.title);
        if let Some(due) = &self.due_date {
            line.push_str(&format!(" 📅 {due}"));
        }
        if let Some(repeat) = &self.recurrence {
            line.push_str(&format!(" 🔁 {repeat}"));
        }
        if let Some(priority) = &self.priority {
            line.push_str(&format!(" {}", priority.marker()));
        }
        line
    }
}

#[derive(Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    fn marker(&self) -> &'static str {
        match self {
            Priority::Low => "⬇️",
            Priority::Medium => "⏫",
            Priority::High => "🔥",
        }
    }
}

pub enum TaskFilter {
    All,
    Open,
    Done,
}

pub struct TaskEntry {
    pub id: usize,
    pub done: bool,
    pub title: String,
    pub raw: String,
}

struct TaskRecord {
    id: usize,
    done: bool,
    title: String,
    raw_text: String,
}

impl TaskRecord {
    fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if !trimmed.starts_with("- [") {
            return None;
        }
        let done_char = trimmed.chars().nth(3)?;
        let done = matches!(done_char, 'x' | 'X');
        let (_, after_raw) = trimmed.split_once(']')?;
        let after_bracket = after_raw.trim();
        if !after_bracket.starts_with('(') {
            return None;
        }
        let end_idx = after_bracket.find(')')?;
        let id_str = &after_bracket[1..end_idx];
        let id = id_str.parse::<usize>().ok()?;
        let remainder = after_bracket[end_idx + 1..].trim();
        let title = extract_title(remainder);
        Some(TaskRecord {
            id,
            done,
            title,
            raw_text: trimmed.to_string(),
        })
    }

    fn into_status_line(self, done: bool) -> String {
        if done {
            let updated = self.raw_text.replacen("[ ]", "[x]", 1);
            if updated != self.raw_text {
                updated
            } else {
                self.raw_text.replacen("[X]", "[x]", 1)
            }
        } else {
            let updated = self.raw_text.replacen("[x]", "[ ]", 1);
            if updated != self.raw_text {
                updated
            } else {
                self.raw_text.replacen("[X]", "[ ]", 1)
            }
        }
    }

    fn into_entry(self) -> TaskEntry {
        TaskEntry {
            id: self.id,
            done: self.done,
            title: self.title,
            raw: self.raw_text,
        }
    }
}

fn extract_title(input: &str) -> String {
    let mut end = input.len();
    for marker in ["📅", "🔁", "⬇️", "⏫", "🔥"] {
        if let Some(idx) = input.find(marker) {
            end = end.min(idx);
        }
    }
    input[..end].trim().to_string()
}
