use clap::{Args, Subcommand, ValueEnum};

use crate::config::AppContext;
use crate::core::tasks::{TaskFilter, TaskService};

#[derive(Subcommand)]
pub enum TaskCommand {
    /// Add a new task to the vault.
    Add(TaskAddArgs),
    /// Mark an existing task as complete.
    Done(TaskDoneArgs),
    /// List tasks, optionally filtered by status.
    List(TaskListArgs),
    /// Remove completed tasks from the task list.
    Clean,
}

#[derive(Args)]
pub struct TaskAddArgs {
    /// Task description.
    #[arg(required = true)]
    pub title: Vec<String>,
    /// Optional due date in YYYY-MM-DD format.
    #[arg(long)]
    pub due: Option<String>,
    /// Optional recurrence string (e.g., weekly).
    #[arg(long)]
    pub repeat: Option<String>,
    /// Optional priority marker (low, medium, high).
    #[arg(long, value_enum)]
    pub priority: Option<TaskPriority>,
}

#[derive(ValueEnum, Clone)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Args)]
pub struct TaskDoneArgs {
    /// Task identifier to mark as done.
    pub id: usize,
}

#[derive(Args)]
pub struct TaskListArgs {
    /// Filter tasks by completion status.
    #[arg(long, value_enum)]
    pub status: Option<TaskStatus>,
}

#[derive(ValueEnum, Clone)]
pub enum TaskStatus {
    Open,
    Done,
}

pub fn handle(cmd: TaskCommand, ctx: &AppContext) -> anyhow::Result<()> {
    let service = TaskService::new(ctx)?;
    match cmd {
        TaskCommand::Add(args) => {
            let new_task = crate::core::tasks::NewTask {
                title: args.title.join(" "),
                due_date: args.due.clone(),
                recurrence: args.repeat.clone(),
                priority: args.priority.map(|p| match p {
                    TaskPriority::Low => crate::core::tasks::Priority::Low,
                    TaskPriority::Medium => crate::core::tasks::Priority::Medium,
                    TaskPriority::High => crate::core::tasks::Priority::High,
                }),
            };
            let id = service.add_task(new_task)?;
            println!("Added task #{id}");
        }
        TaskCommand::Done(args) => {
            service.mark_done(args.id)?;
            println!("Marked task #{} as done", args.id);
        }
        TaskCommand::List(args) => {
            let filter = match args.status {
                Some(TaskStatus::Open) => TaskFilter::Open,
                Some(TaskStatus::Done) => TaskFilter::Done,
                None => TaskFilter::All,
            };
            for rendered in service.list_tasks(filter)? {
                println!("{rendered}");
            }
        }
        TaskCommand::Clean => {
            service.clean_completed()?;
        }
    }
    Ok(())
}
