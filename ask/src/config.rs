use crate::Cli;
use crate::consts::*;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Config {
    pub model: String,
    pub url: String,
    pub key: String,
    pub system_prompt: String,
    pub history_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "SOME_MODEL".into(),
            url: "https://api.deepseek.com/chat/completions".into(),
            key: "sk-xxx".into(),
            system_prompt: SYSTEM_PROMPT.into(),
            history_path: expand_tilde("~/.ask_history").display().to_string(),
        }
    }
}
pub fn expand_tilde(path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.starts_with("~") {
        if let Some(home) = std::env::home_dir() {
            if let Ok(stripped) = path.strip_prefix("~") {
                return home.join(stripped);
            }
        }
    }
    path.to_path_buf()
}

pub fn load_config(cli: &Cli) -> Config {
    let mut cfg: Config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file(expand_tilde(CONFIG_PATH)))
        .merge(Env::prefixed("APP_").split("_"))
        .extract()
        .expect("Can't load config file");
    if cli.url.is_some() {
        cfg.url = cli.url.clone().unwrap();
    }
    if cli.model.is_some() {
        cfg.model = cli.model.clone().unwrap();
    }
    if cli.key.is_some() {
        cfg.key = cli.key.clone().unwrap();
    }
    if cli.system_prompt.is_some() {
        cfg.system_prompt = cli.system_prompt.clone().unwrap();
    }
    if cli.history_path.is_some() {
        cfg.history_path = cli.history_path.clone().unwrap();
    }
    cfg
}
