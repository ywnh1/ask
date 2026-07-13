// SPDX-License-Identifier: MIT
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Local;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use streamdown_parser::Parser as SdParser;
use streamdown_render::Renderer as SdRenderer;
use streamdown_render::terminal_width;
use tokio::fs;
use tokio::io::AsyncWriteExt;

// ─── API 数据结构 ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

// ─── 历史记录解析 ───────────────────────────────────────────────

#[derive(Debug)]
struct HistoryEntry {
    message: Option<String>,
    response: Option<String>,
}

/// 解析历史文件内容，提取每一条对话记录
fn parse_history(content: &str) -> Vec<HistoryEntry> {
    let mut entries = Vec::new();

    for block in content.split("---") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let mut entry = HistoryEntry {
            message: None,
            response: None,
        };

        for line in block.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("message:") {
                entry.message = Some(val.trim().to_string());
            } else if let Some(val) = line.strip_prefix("response:") {
                entry.response = Some(val.trim().to_string());
            }
        }

        entries.push(entry);
    }

    entries
}

// ─── Ai 主结构 ──────────────────────────────────────────────────

pub struct Ai {
    pub url: String,
    pub key: String,
    pub model: String,
    pub system_prompt: String,
    pub messages: String,
    pub history_path: Option<String>,
    pub response: Option<String>,
}

impl Ai {
    pub fn new(
        url: &str,
        key: &str,
        model: &str,
        sys: &str,
        msg: &str,
        history: Option<&str>,
    ) -> Self {
        let history_path = if let Some(history) = history {
            Some(String::from(history))
        } else {
            None
        };
        Self {
            url: String::from(url),
            key: String::from(key),
            model: String::from(model),
            system_prompt: String::from(sys),
            messages: String::from(msg),
            history_path,
            response: None,
        }
    }
    /// 执行一次完整的 AI 对话
    ///
    /// 1. 若 `history_path` 有值且文件不存在，则创建
    /// 2. 构建消息列表（system_prompt + 历史记录 + 当前 messages）
    /// 3. 以流式方式发送请求，逐字输出到 stdout
    /// 4. 将完整响应保存到 `self.response`
    pub async fn work(&mut self) -> Result<()> {
        // 1. 检查历史文件
        if let Some(path) = &self.history_path {
            let p = Path::new(path);
            if !p.exists() {
                if let Some(parent) = p.parent() {
                    fs::create_dir_all(parent).await?;
                }
                fs::write(p, "").await?;
            }
        }

        // 2. 构建消息列表
        let mut msgs: Vec<ChatMessage> = Vec::new();

        if !self.system_prompt.is_empty() {
            msgs.push(ChatMessage {
                role: "system".into(),
                content: self.system_prompt.clone(),
            });
        }

        // 加载历史记录
        if let Some(path) = &self.history_path {
            let content = fs::read_to_string(path).await?;
            if !content.trim().is_empty() {
                for entry in parse_history(&content) {
                    if let Some(msg) = entry.message {
                        msgs.push(ChatMessage {
                            role: "user".into(),
                            content: msg,
                        });
                    }
                    if let Some(resp) = entry.response {
                        msgs.push(ChatMessage {
                            role: "assistant".into(),
                            content: resp,
                        });
                    }
                }
            }
        }

        // 当前用户消息
        msgs.push(ChatMessage {
            role: "user".into(),
            content: self.messages.clone(),
        });

        // 3. 发送请求
        let client = Client::new();
        let req_body = ChatRequest {
            model: self.model.clone(),
            messages: msgs,
            stream: true,
        };

        let resp = client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.key))
            .json(&req_body)
            .send()
            .await
            .context("请求发送失败，请检查网络或 URL")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await?;
            return Err(anyhow::anyhow!("API 返回错误 [{status}]: {body}"));
        }

        // 4. 流式读取并使用 Streamdown 渲染 Markdown
        let mut full = String::new();
        let mut buf = String::new();
        let mut content_buf = String::new();
        let mut stream = resp.bytes_stream();

        let width = terminal_width();
        let mut sd_parser = SdParser::new();
        let mut sd_renderer = SdRenderer::new(std::io::stdout(), width);

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("读取流数据失败")?;
            buf.push_str(&String::from_utf8_lossy(&chunk));

            // 逐行处理 SSE 数据
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim_end_matches('\r').to_string();
                buf.drain(..=pos);

                if let Some(data) = line.strip_prefix("data: ") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<StreamResponse>(data) {
                        for choice in parsed.choices {
                            if let Some(c) = choice.delta.content {
                                full.push_str(&c);
                                content_buf.push_str(&c);

                                // 逐行通过 Streamdown 增量渲染
                                while let Some(pos) = content_buf.find('\n') {
                                    let line_content = content_buf[..pos].to_string();
                                    content_buf.drain(..=pos);
                                    let events = sd_parser.parse_line(&line_content);
                                    sd_renderer
                                        .render(&events)
                                        .context("Streamdown 渲染失败")?;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 处理没有换行结尾的剩余内容
        if !content_buf.is_empty() {
            let events = sd_parser.parse_line(&content_buf);
            sd_renderer
                .render(&events)
                .context("Streamdown 渲染失败")?;
        }

        // 结束解析，关闭所有打开的块
        let events = sd_parser.finalize();
        sd_renderer
            .render(&events)
            .context("Streamdown 渲染失败")?;

        // 换到新行
        println!();

        self.response = Some(full);
        Ok(())
    }

    /// 将本次对话追加到历史文件
    ///
    /// 输出格式：
    /// ```text
    /// ---
    /// time: YY.MM.DD hh:mm:ss
    /// model: ...
    /// message: ...
    /// response: ...
    /// ```
    pub async fn history(&self) -> Result<()> {
        let path = self
            .history_path
            .as_ref()
            .context("history_path 未设置，无法写入历史")?;

        let now = Local::now().format("%y.%m.%d %H:%M:%S");
        let resp = self.response.as_deref().unwrap_or("(无响应)");

        let entry = format!(
            "---\ntime: {now}\nmodel: {model}\nmessage: {msg}\nresponse: {resp}\n",
            model = self.model,
            msg = self.messages,
        );

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        file.write_all(entry.as_bytes()).await?;
        Ok(())
    }
}
