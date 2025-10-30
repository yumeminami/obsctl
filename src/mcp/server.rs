use std::{fs, sync::Arc};

use anyhow::Result as AnyResult;
use rmcp::{
    handler::server::{tool::parse_json_object, ServerHandler},
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, JsonObject, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
        ToolAnnotations,
    },
    ErrorData as McpError,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    config::AppContext,
    core::{
        tasks::{TaskEntry, TaskService},
        vault::VaultService,
    },
    search::SearchService,
};

#[derive(Clone)]
pub struct ObsctlMcpServer {
    ctx: Arc<AppContext>,
}

impl ObsctlMcpServer {
    pub fn new(ctx: AppContext) -> Self {
        Self { ctx: Arc::new(ctx) }
    }

    fn list_available_tools(&self) -> Vec<Tool> {
        vec![
            append_daily_tool(),
            update_task_tool(),
            query_knowledge_tool(),
            summarize_today_tool(),
        ]
    }

    fn append_daily(&self, entry: &str) -> AnyResult<String> {
        let vault = VaultService::new(&self.ctx)?;
        vault.append_today(entry)?;
        Ok(format!(
            "Appended entry to {}",
            vault.today_path().display()
        ))
    }

    fn update_task_status(&self, params: UpdateTaskStatusParams) -> Result<String, McpError> {
        let status_flag = normalize_status(&params.status)?;
        if params.id.is_none() && params.title.is_none() {
            return Err(McpError::invalid_params(
                "either `id` or `title` must be provided",
                None,
            ));
        }

        let service =
            TaskService::new(&self.ctx).map_err(|err| internal_error("load tasks", err))?;

        let target: Option<TaskEntry> = if let Some(id) = params.id {
            service
                .tasks()
                .map_err(|err| internal_error("read tasks", err))?
                .into_iter()
                .find(|task| task.id == id)
        } else if let Some(title) = params.title.as_ref() {
            service
                .find_task_by_title(title)
                .map_err(|err| internal_error("search task by title", err))?
        } else {
            None
        };

        let task = target.ok_or_else(|| {
            McpError::invalid_params(
                "task not found",
                Some(json!({ "id": params.id, "title": params.title })),
            )
        })?;

        service
            .set_status(task.id, status_flag)
            .map_err(|err| internal_error("update task status", err))?;

        let status_label = if status_flag { "done" } else { "open" };
        Ok(format!("Task #{} marked as {}", task.id, status_label))
    }

    fn query_knowledge(&self, params: QueryKnowledgeParams) -> Result<String, McpError> {
        let limit = params.limit.unwrap_or(5).max(1);
        let service =
            SearchService::new(&self.ctx).map_err(|err| internal_error("init search", err))?;
        let matches = service
            .grep_matches(&params.query, limit)
            .map_err(|err| internal_error("run search", err))?;
        if matches.is_empty() {
            Ok(format!("No matches for \"{}\"", params.query))
        } else {
            Ok(matches.join("\n"))
        }
    }

    fn summarize_today(&self) -> Result<String, McpError> {
        let vault =
            VaultService::new(&self.ctx).map_err(|err| internal_error("load vault", err))?;
        let path = vault
            .path_for(None)
            .map_err(|err| internal_error("locate daily note", err))?;
        let content = fs::read_to_string(&path)
            .map_err(|err| internal_error("read daily note", err.into()))?;
        let summary = summarize_text(&content);
        Ok(format!(
            "{}\n{}\n{}",
            path.display(),
            "-".repeat(40),
            summary
        ))
    }
}

impl ServerHandler for ObsctlMcpServer {
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        let tools = self.list_available_tools();
        async move {
            Ok(ListToolsResult {
                tools,
                next_cursor: None,
            })
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let CallToolRequestParam { name, arguments } = request;
        match name.as_ref() {
            "append_daily_note" => {
                let params: AppendDailyNoteParams =
                    parse_json_object(arguments.clone().unwrap_or_default())?;
                let message = self
                    .append_daily(&params.entry)
                    .map_err(|err| internal_error("append daily note", err))?;
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            "update_task_status" => {
                let params: UpdateTaskStatusParams =
                    parse_json_object(arguments.clone().unwrap_or_default())?;
                let message = self.update_task_status(params)?;
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            "query_knowledge" => {
                let params: QueryKnowledgeParams =
                    parse_json_object(arguments.clone().unwrap_or_default())?;
                let body = self.query_knowledge(params)?;
                Ok(CallToolResult::success(vec![Content::text(body)]))
            }
            "summarize_today" => {
                let params: SummarizeTodayParams =
                    parse_json_object(arguments.unwrap_or_default())?;
                if let Some(scope) = params.scope {
                    if scope.to_lowercase() != "today" {
                        return Err(McpError::invalid_params(
                            "scope must be \"today\"",
                            Some(json!({ "scope": scope })),
                        ));
                    }
                }
                let summary = self.summarize_today()?;
                Ok(CallToolResult::success(vec![Content::text(summary)]))
            }
            other => Err(McpError::invalid_params(
                format!("unknown tool: {other}"),
                Some(json!({ "tool": other })),
            )),
        }
    }

    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_tool_list_changed()
            .build();
        let mut info = Implementation::from_build_env();
        info.title = Some("obsctl MCP server".to_string());
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities,
            server_info: info,
            instructions: Some(
                "Tools expose daily note append, task updates, search, and summaries.".to_string(),
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AppendDailyNoteParams {
    entry: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTaskStatusParams {
    #[serde(default)]
    id: Option<usize>,
    #[serde(default)]
    title: Option<String>,
    status: String,
}

#[derive(Debug, Deserialize)]
struct QueryKnowledgeParams {
    query: String,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
struct SummarizeTodayParams {
    #[serde(default)]
    scope: Option<String>,
}

fn append_daily_tool() -> Tool {
    let schema = json!({
        "type": "object",
        "properties": {
            "entry": {
                "type": "string",
                "description": "Freeform text that will be appended to today's daily note."
            }
        },
        "required": ["entry"],
    });
    Tool::new(
        "append_daily_note",
        "Append text to today's daily note",
        schema_arc(schema),
    )
    .annotate(
        ToolAnnotations::with_title("Append Daily Note")
            .destructive(false)
            .idempotent(false),
    )
}

fn update_task_tool() -> Tool {
    let schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "integer", "minimum": 1 },
            "title": { "type": "string" },
            "status": {
                "type": "string",
                "enum": ["done", "completed", "complete", "open", "todo", "pending"]
            }
        },
        "required": ["status"],
        "anyOf": [
            { "required": ["id"] },
            { "required": ["title"] }
        ]
    });
    Tool::new(
        "update_task_status",
        "Mark a task as done or reopen it",
        schema_arc(schema),
    )
    .annotate(
        ToolAnnotations::with_title("Update Task Status")
            .destructive(false)
            .idempotent(true),
    )
}

fn query_knowledge_tool() -> Tool {
    let schema = json!({
        "type": "object",
        "properties": {
            "query": { "type": "string" },
            "limit": {
                "type": "integer",
                "minimum": 1,
                "maximum": 50,
                "default": 5
            }
        },
        "required": ["query"]
    });
    Tool::new(
        "query_knowledge",
        "Search the vault for matching lines",
        schema_arc(schema),
    )
    .annotate(
        ToolAnnotations::with_title("Query Knowledge")
            .read_only(true)
            .idempotent(true),
    )
}

fn summarize_today_tool() -> Tool {
    let schema = json!({
        "type": "object",
        "properties": {
            "scope": {
                "type": "string",
                "enum": ["today"],
                "default": "today"
            }
        }
    });
    Tool::new(
        "summarize_today",
        "Summarize today's daily note in plain text",
        schema_arc(schema),
    )
    .annotate(
        ToolAnnotations::with_title("Summarize Today")
            .read_only(true)
            .idempotent(true),
    )
}

fn schema_arc(value: serde_json::Value) -> Arc<JsonObject> {
    Arc::new(rmcp::model::object(value))
}

fn internal_error(context: &str, err: anyhow::Error) -> McpError {
    McpError::internal_error(
        format!("{context} failed"),
        Some(json!({ "error": err.to_string() })),
    )
}

fn normalize_status(input: &str) -> Result<bool, McpError> {
    match input.to_lowercase().as_str() {
        "done" | "complete" | "completed" | "finished" => Ok(true),
        "open" | "todo" | "pending" | "reopen" | "reopened" => Ok(false),
        other => Err(McpError::invalid_params(
            format!("unknown status: {other}"),
            Some(json!({ "status": other })),
        )),
    }
}

fn summarize_text(content: &str) -> String {
    let mut highlights = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            highlights.push(trimmed.to_string());
        }
        if highlights.len() >= 12 {
            break;
        }
    }
    if highlights.is_empty() {
        "No notable entries found for today.".to_string()
    } else {
        highlights.join("\n")
    }
}
