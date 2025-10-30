# obsctl

本地优先的命令行知识管理工具，支持通过 MCP 接口与本地 AI 协作，灵感来自 Obsidian 的工作流。

## 项目愿景
- 终端里即可快速记录每日笔记。
- 以 Markdown 形式管理任务，支持 Due/Recurring/Priority 属性。
- 通过 ripgrep / fzf 实现本地全文检索，并预留 RAG 拓展。
- 提供 MCP 函数，让本地 LLM 可以总结笔记或更新任务。

## 快速开始

```bash
cargo run -- note add "调试 MPC 控制器"
cargo run -- task add "整理项目周报" --due 2025-01-05 --priority high
cargo run -- search grep "torque mapping"
```

部分命令依赖外部工具，请先确保已安装：

- `rg`（ripgrep，用于全文搜索）：macOS `brew install ripgrep`，Ubuntu `sudo apt install ripgrep`
- `fzf`（模糊查找）：macOS `brew install fzf`，Ubuntu `sudo apt install fzf`

## 安装方式

可直接通过 GitHub 获取最新发行版：

```bash
curl -fsSL https://raw.githubusercontent.com/yumeminami/obsctl/master/scripts/install.sh | sh
```

如果需要指定安装目录，可预先设置 `OBSCTL_INSTALL_DIR`（默认安装至 `~/.local/bin`）。
脚本会根据当前系统自动选择 macOS / Linux 以及 x86-64 / ARM64 的对应压缩包，并同时安装
`obsctl` 与配套的 `obsctl_mcp` 服务端可执行文件。

首次运行建议执行：

```bash
cargo run -- config init
```

将自动创建 `~/.obsctl/` 目录并初始化 Vault 结构：

```
~/.obsctl/
├── config.toml
└── vault/
    ├── Journal/
    ├── Tasks/tasks.md
    ├── Projects/
    └── templates/
        ├── daily.md
        └── task.md
```

如需更换 Vault 路径，可编辑 `config.toml` 或执行 `cargo run -- config path --set <路径>`。

## 命令概览

- `note add|open|list`：追加每日笔记、查看指定日期、列出最近记录。
- `task add|done|list|clean`：新增、完成、筛选、清理任务，支持 Due / 🔁 / 优先级标记。
- `search grep|fzf`：利用 ripgrep 全文搜索或 fzf 文件模糊查找。
- `config init|path`：初始化配置，查看或更新 Vault 路径。
- `version [--json|--verbose]`：输出当前版本信息，支持 JSON 与详细模式。

更多参数说明可执行 `cargo run -- --help` 查看。

## MCP 服务

- 运行 `cargo run --bin obsctl_mcp` 启动基于 stdio 的 MCP Server。
- 提供工具：`append_daily_note`、`update_task_status`、`query_knowledge`、`summarize_today`。
- 基于官方 `rmcp` Rust SDK，实现与本地 LLM/Agent 的 MCP 协议通信。
- 可让 AI 自动补充每日笔记、更新任务状态、执行知识检索。

## 更新日志

查看 [`CHANGELOG.md`](CHANGELOG.md) 了解版本变更记录。

## 代码结构

- `src/cli`：基于 clap 的命令解析与处理。
- `src/config`：加载/保存 TOML 配置，确保 Vault 目录存在。
- `src/core`：笔记 (`vault`) 与任务 (`tasks`) 服务层。
- `src/search`：调用 ripgrep / fzf 的搜索封装。
- `src/mcp`：基于 rmcp SDK 的 MCP 服务实现。
- `src/templates`：默认的每日笔记与任务模板内容。

服务层负责处理文件读写，CLI 与未来的 MCP 调用都可以复用。

## 开发说明

```bash
cargo check
cargo fmt
cargo test
```

- 若使用 pre-commit，可运行 `pre-commit install` 启用 `.pre-commit-config.yaml` 中的 `cargo fmt`/`cargo clippy` 钩子。

后续规划包括：接入真实 MCP 客户端、构建向量/RAG 索引、以及更丰富的 AI 协作能力。
