// SPDX-License-Identifier: MIT
mod config;
mod consts;
mod format;
use crate::format::{FormatTarget, IntoMsg};
use anyhow::Result;
use clap::Parser;
use connect::Ai;
use consts::*;
use std::{fs, io::Write, path::Path};
use toml::to_string_pretty;
#[tokio::main]
async fn main() {
    if let Err(err) = main_process().await {
        let chain: Vec<_> = err.chain().map(|e| e.to_string()).collect();
        eprintln!("The error chain's length: {}", chain.len());
        for (i, msg) in chain.iter().enumerate() {
            eprintln!("[{}] {}", i + 1, msg);
        }
        std::process::exit(1);
    }
}

async fn main_process() -> Result<()> {
    let cp = config::expand_tilde(CONFIG_PATH);
    let cfg_path = Path::new(&cp);
    if !cfg_path.is_file() {
        fs::create_dir_all(
            cfg_path
                .parent()
                .expect(&format!("Fail to create {CONFIG_PATH}")),
        )?;
        let mut cfg = fs::File::create(cfg_path)?;
        let cfg_toml = to_string_pretty(&config::Config::default())?;
        cfg.write_all(&cfg_toml.into_bytes())?;
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        std::process::Command::new(&editor).arg(cp).status()?;
        return Ok(());
    }
    let cli = Cli::parse();
    let config = config::load_config(&cli);
    let history = if cli.disable_history {
        None
    } else {
        Some(config.history_path.as_str())
    };
    let mut msg = cli.msg.into_msg(FormatTarget::None)?;
    if !cli.file.is_empty() {
        msg.push_str(&cli.file.into_msg(FormatTarget::ToString)?);
    }
    if !cli.format.is_empty() {
        msg.push_str(&cli.format.into_msg(FormatTarget::ToMd)?);
    }
    if !cli.base64.is_empty() {
        msg.push_str(&cli.base64.into_msg(FormatTarget::ToBase64)?)
    }
    let mut ai = Ai::new(
        &config.url,
        &config.key,
        &config.model,
        &config.system_prompt,
        &msg,
        history,
    );
    println!("=== {} ===", config.model);
    ai.work().await?;
    if !cli.disable_history {
        ai.history().await?;
    }

    if cli.clear_history {
        fs::remove_file(&config.history_path)?;
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(version, name = "ask")]
#[command(
    about = "Chat with AI from your terminal",
    long_about = "A minimal CLI tool that lets you chat with AI models \
(DeepSeek, OpenAI-compatible APIs) directly from your shell.\n\
Supports streaming responses, conversation history, and file context \
(plaintext, markdown, base64)."
)]
struct Cli {
    /// Message to send to the AI
    msg: Vec<String>,
    /// Attach a file as plaintext
    #[arg(short, long)]
    file: Vec<String>,
    /// Attach a file converted to markdown
    #[arg(short = 'F', long)]
    format: Vec<String>,
    /// Attach a file as base64
    #[arg(short, long)]
    base64: Vec<String>,
    /// Disable conversation history
    #[arg(short, long)]
    disable_history: bool,
    /// Override the API URL
    #[arg(long)]
    url: Option<String>,
    /// Override the model name
    #[arg(long)]
    model: Option<String>,
    /// Override the API key
    #[arg(long)]
    key: Option<String>,
    /// Override the system prompt
    #[arg(long)]
    system_prompt: Option<String>,
    /// Override the history file path
    #[arg(long)]
    history_path: Option<String>,
    /// Delete the history file
    #[arg(long)]
    clear_history: bool,
}
