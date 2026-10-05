# ask

[![Rust](https://img.shields.io/badge/rust-2024-dea584?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![中文](https://img.shields.io/badge/lang-中文-red)](README_zh.md)

> Chat with AI from your terminal.

**ask** is a Rust CLI that lets you talk to AI models (DeepSeek, OpenAI-compatible APIs) directly from your shell — streaming responses, conversation history, file attachments, all in one binary.

Two crates, one tool:

- **`ask`** — CLI binary: argument parsing, config loading, file formatting.
- **`connect`** — library: AI API client, SSE streaming, history persistence.

## Quick start

```bash
# Install Rust, then:
git clone https://github.com/ywnh1/ask.git
cd ask/ask
cargo build --release
cp target/release/ask ~/.local/bin/

# Configure (opens editor on first run)
ask --help
```

## Features

| | |
|---|---|
| 💬 | Natural questions: `ask What is Rust?` |
| 📁 | Attach files as plaintext, markdown, or base64 |
| 🔄 | Streaming responses with real-time Markdown rendering |
| 📜 | Conversation history (`~/.ask_history`) |
| ⚙️ | Config via TOML or `APP_*` environment variables |
| 🧩 | Override any setting per-request with CLI flags |

## Configuration

On first launch, `ask` opens `~/.config/ask/config.toml` in your editor:

```toml
model = "deepseek-flash"
url = "https://api.deepseek.com/chat/completions"
key = "sk-your-api-key"
system_prompt = "You are a helpful AI."
history_path = "~/.ask_history"
```

Environment variables (prefixed with `APP_`, split by `_`):

```bash
export APP_MODEL="deepseek-chat"
export APP_KEY="sk-xxx"
```

**Priority:** CLI flags > config file > env vars > defaults.

## Usage

```bash
# Ask anything
ask What is Rust?
ask "Explain async/await in Rust"

# Attach files
ask Summarize this file       -f notes.txt
ask What format is this       -F report.docx
ask What is in this image     -b screenshot.png

# History control
ask --disable-history Tell me a joke
ask --clear-history
```

### CLI reference

| Flag | Description |
|---|---|
| `[MSG]...` | Message to send to the AI |
| `-f, --file <FILE>` | Attach a file as plaintext |
| `-F, --format <FILE>` | Attach a file converted to markdown |
| `-b, --base64 <FILE>` | Attach a file as base64 |
| `-d, --disable-history` | Disable conversation history |
| `--url <URL>` | Override the API URL |
| `--model <MODEL>` | Override the model name |
| `--key <KEY>` | Override the API key |
| `--system-prompt <PROMPT>` | Override the system prompt |
| `--history-path <PATH>` | Override the history file path |
| `--clear-history` | Delete the history file |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## License

MIT
