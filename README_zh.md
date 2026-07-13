# ask

[![Rust](https://img.shields.io/badge/rust-2024-dea584?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![English](https://img.shields.io/badge/lang-EN-blue)](README.md)

> 在终端中与 AI 对话。

**ask** 是一个 Rust 命令行工具，让你在 Shell 中直接与 AI 模型（DeepSeek、兼容 OpenAI 的 API）对话——流式响应、对话历史、文件附件，一个二进制全搞定。

两个 crate，一个工具：

- **`ask`** — CLI 主程序：参数解析、配置加载、文件格式化。
- **`connect`** — 库：AI API 客户端、SSE 流式读取、历史记录持久化。

## 快速开始

```bash
# 安装 Rust 后：
git clone https://github.com/yourusername/ask.git
cd ask/ask
cargo build --release
cp target/release/ask ~/.local/bin/

# 首次运行会自动生成配置并打开编辑器
ask --help
```

## 功能特性

| | |
|---|---|
| 💬 | 自然提问：`ask What is Rust?` |
| 📁 | 附加文件为纯文本、Markdown 或 Base64 |
| 🔄 | 流式响应，实时 Markdown 渲染 |
| 📜 | 对话历史记录（`~/.ask_history`） |
| ⚙️ | 通过 TOML 或 `APP_*` 环境变量配置 |
| 🧩 | 每次请求可通过 CLI 参数覆盖任意设置 |

## 配置

首次启动时，`ask` 会自动打开 `~/.config/ask/config.toml` 供你编辑：

```toml
model = "deepseek-chat"
url = "https://api.deepseek.com/chat/completions"
key = "sk-your-api-key"
system_prompt = "You are a helpful AI."
history_path = "~/.ask_history"
```

环境变量（前缀 `APP_`，以下划线分割）：

```bash
export APP_MODEL="deepseek-chat"
export APP_KEY="sk-xxx"
```

**优先级：** CLI 参数 > 配置文件 > 环境变量 > 默认值。

## 使用方式

```bash
# 直接提问
ask What is Rust?
ask "Explain async/await in Rust"

# 附加文件
ask Summarize this file         -f notes.txt
ask What format is this         -F report.docx
ask What is in this image       -b screenshot.png

# 历史记录控制
ask --disable-history 给我讲个笑话
ask --clear-history
```

### CLI 选项参考

| 参数 | 说明 |
|---|---|
| `[MSG]...` | 发送给 AI 的消息 |
| `-f, --file <FILE>` | 附加纯文本文件 |
| `-F, --format <FILE>` | 附加文件并转为 Markdown |
| `-b, --base64 <FILE>` | 附加文件并转为 Base64 |
| `-d, --disable-history` | 禁用对话历史 |
| `--url <URL>` | 临时覆盖 API 地址 |
| `--model <MODEL>` | 临时覆盖模型名称 |
| `--key <KEY>` | 临时覆盖 API 密钥 |
| `--system-prompt <PROMPT>` | 临时覆盖系统提示词 |
| `--history-path <PATH>` | 临时覆盖历史文件路径 |
| `--clear-history` | 删除历史文件 |
| `-h, --help` | 打印帮助信息 |
| `-V, --version` | 打印版本信息 |

## 开源协议

MIT
