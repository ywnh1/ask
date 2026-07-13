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
#[command(about = "A simple CLI tool which can be used to ask ai questions quickly", long_about = None)]
struct Cli {
    /// The message you want to say to your AI.
    /// It could be used like `ask What is Rust` or `ask "What is Rust"`.
    msg: Vec<String>,
    /// To send AI a plaintext.
    /// It could be used like `ask What are the files about -f a.md`.
    #[arg(short, long)]
    file: Vec<String>,
    /// To send AI a file that can be turned into markdown.
    /// It could be used like `ask What are the files about -F a.docx`.
    #[arg(short = 'F', long)]
    format: Vec<String>,
    /// To send AI a file which will be turned into base64.
    /// It could be used like `ask What is the picture about -b a.jpg`
    #[arg(short, long)]
    base64: Vec<String>,
    /// To disable the ai to read and write the history file.
    #[arg(short, long)]
    disable_history: bool,
    /// To set url once.
    #[arg(long)]
    url: Option<String>,
    /// To set model once.
    #[arg(long)]
    model: Option<String>,
    /// To set key once.
    #[arg(long)]
    key: Option<String>,
    /// To set system prompt once.
    #[arg(long)]
    system_prompt: Option<String>,
    /// To set history file's path once.
    #[arg(long)]
    history_path: Option<String>,
    /// To delete history file
    #[arg(long)]
    clear_history: bool,
}
