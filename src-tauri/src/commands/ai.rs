use crate::AppState;
use crate::commands::types::*;
use crate::keychain::Keychain;
use tauri::State;
use tauri::ipc::Channel;
use reqwest::Client;
use rusqlite::params;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;
use futures_util::StreamExt;
use log::info;

fn get_ollama_base_url(state: &AppState) -> String {
    if let Ok(db) = state.db.lock() {
        let result: Result<String, _> = db.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params!["ollama_base_url"],
            |row| row.get(0),
        );
        if let Ok(url) = result {
            let trimmed = url.trim().trim_end_matches('/').to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    "http://localhost:11434".to_string()
}

fn interpolate_variables(prompt: &str, variables: &Option<HashMap<String, String>>) -> String {
    let mut result = prompt.to_string();
    if let Some(vars) = variables {
        for (key, value) in vars {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
    }
    result
}

/// Prepend all non-image attachments as labelled file blocks before the prompt text.
fn prepend_text_attachments(prompt: &str, attachments: &Option<Vec<crate::commands::types::Attachment>>) -> String {
    let atts = match attachments {
        Some(v) => v,
        None => return prompt.to_string(),
    };
    let text_blocks: Vec<String> = atts.iter()
        .filter(|a| !a.mime_type.starts_with("image/"))
        .map(|a| format!("=== File: {} ===\n{}", a.name, a.content))
        .collect();
    if text_blocks.is_empty() {
        return prompt.to_string();
    }
    format!("{}\n\n---\n\n{}", text_blocks.join("\n\n"), prompt)
}

/// Return only image attachments from the optional list.
fn image_attachments(attachments: &Option<Vec<crate::commands::types::Attachment>>) -> Vec<&crate::commands::types::Attachment> {
    match attachments {
        Some(v) => v.iter().filter(|a| a.mime_type.starts_with("image/")).collect(),
        None => vec![],
    }
}

#[tauri::command]
pub async fn run_prompt(state: State<'_, AppState>, request: RunPromptRequest) -> Result<ProviderResponse, String> {
    let start = Instant::now();
    let interpolated = interpolate_variables(&request.prompt, &request.variables);
    let prompt = prepend_text_attachments(&interpolated, &request.attachments);
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let images = image_attachments(&request.attachments);

    match request.provider.as_str() {
        "openai" => {
            let api_key = Keychain::get_api_key("openai").map_err(|e| e.to_string())?;

            // Build content array: text first, then images
            let mut content_parts: Vec<Value> = vec![json!({"type": "text", "text": prompt})];
            for img in &images {
                content_parts.push(json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:{};base64,{}", img.mime_type, img.content)
                    }
                }));
            }

            let user_content: Value = if images.is_empty() {
                json!(prompt)
            } else {
                json!(content_parts)
            };

            let body = json!({
                "model": request.model,
                "messages": [{"role": "user", "content": user_content}],
                "temperature": request.temperature.unwrap_or(0.7),
                "max_tokens": request.max_tokens.unwrap_or(4096)
            });

            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

            let content = data["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let input_tokens = data["usage"]["prompt_tokens"].as_u64().map(|v| v as u32);
            let output_tokens = data["usage"]["completion_tokens"].as_u64().map(|v| v as u32);
            let duration = start.elapsed().as_millis() as u64;

            Ok(ProviderResponse {
                content,
                model: request.model,
                tokens_input: input_tokens,
                tokens_output: output_tokens,
                duration_ms: Some(duration),
            })
        }
        "anthropic" => {
            let api_key = Keychain::get_api_key("anthropic").map_err(|e| e.to_string())?;

            // Anthropic: images first, then text
            let user_content: Value = if images.is_empty() {
                json!(prompt)
            } else {
                let mut parts: Vec<Value> = images.iter().map(|img| json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": img.mime_type,
                        "data": img.content
                    }
                })).collect();
                parts.push(json!({"type": "text", "text": prompt}));
                json!(parts)
            };

            let body = json!({
                "model": request.model,
                "messages": [{"role": "user", "content": user_content}],
                "max_tokens": request.max_tokens.unwrap_or(4096)
            });

            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

            let content = data["content"][0]["text"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let input_tokens = data["usage"]["input_tokens"].as_u64().map(|v| v as u32);
            let output_tokens = data["usage"]["output_tokens"].as_u64().map(|v| v as u32);
            let duration = start.elapsed().as_millis() as u64;

            Ok(ProviderResponse {
                content,
                model: request.model,
                tokens_input: input_tokens,
                tokens_output: output_tokens,
                duration_ms: Some(duration),
            })
        }
        "ollama" => {
            let base_url = get_ollama_base_url(&state);

            // Ollama supports images via the `images` array (llava, etc.)
            let image_b64s: Vec<&str> = images.iter().map(|img| img.content.as_str()).collect();

            let mut body_map = json!({
                "model": request.model,
                "prompt": prompt,
                "stream": false,
                "options": {
                    "temperature": request.temperature.unwrap_or(0.7),
                    "num_predict": request.max_tokens.unwrap_or(4096)
                }
            });
            if !image_b64s.is_empty() {
                body_map["images"] = json!(image_b64s);
            }

            let response = client
                .post(format!("{}/api/generate", base_url))
                .header("Content-Type", "application/json")
                .json(&body_map)
                .send()
                .await
                .map_err(|e| format!("Cannot reach Ollama at {}. Is it running? ({})", base_url, e))?;

            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

            let content = data["response"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let duration = start.elapsed().as_millis() as u64;

            Ok(ProviderResponse {
                content,
                model: request.model,
                tokens_input: None,
                tokens_output: None,
                duration_ms: Some(duration),
            })
        }
        "gemini" => {
            let api_key = Keychain::get_api_key("gemini").map_err(|e| e.to_string())?;

            let mut parts: Vec<Value> = vec![json!({"text": prompt})];
            for img in &images {
                parts.push(json!({
                    "inline_data": {
                        "mime_type": img.mime_type,
                        "data": img.content
                    }
                }));
            }

            let body = json!({
                "contents": [{"parts": parts}],
                "generationConfig": {
                    "temperature": request.temperature.unwrap_or(0.7),
                    "maxOutputTokens": request.max_tokens.unwrap_or(4096)
                }
            });

            let response = client
                .post(&format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                    request.model, api_key
                ))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

            let content = data["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let duration = start.elapsed().as_millis() as u64;

            Ok(ProviderResponse {
                content,
                model: request.model,
                tokens_input: None,
                tokens_output: None,
                duration_ms: Some(duration),
            })
        }
        "mistral" => {
            let api_key = Keychain::get_api_key("mistral").map_err(|e| e.to_string())?;

            // Mistral doesn't support images — text-only
            let body = json!({
                "model": request.model,
                "messages": [{"role": "user", "content": prompt}],
                "temperature": request.temperature.unwrap_or(0.7),
                "max_tokens": request.max_tokens.unwrap_or(4096)
            });

            let response = client
                .post("https://api.mistral.ai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

            let content = data["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let input_tokens = data["usage"]["prompt_tokens"].as_u64().map(|v| v as u32);
            let output_tokens = data["usage"]["completion_tokens"].as_u64().map(|v| v as u32);
            let duration = start.elapsed().as_millis() as u64;

            Ok(ProviderResponse {
                content,
                model: request.model,
                tokens_input: input_tokens,
                tokens_output: output_tokens,
                duration_ms: Some(duration),
            })
        }
        _ => Err(format!("Unknown provider: {}", request.provider)),
    }
}

// ─── Real streaming via Tauri Channel ────────────────────────────────────────

/// Helper: parse an SSE byte stream (OpenAI / Mistral / Anthropic / Gemini format)
/// and call the provided closure with each complete `data:` line payload.
async fn consume_sse<F>(
    response: reqwest::Response,
    mut on_data: F,
) -> Result<(), String>
where
    F: FnMut(&str) -> Result<(), String>,
{
    let mut stream = response.bytes_stream();
    let mut buf = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buf.push_str(&String::from_utf8_lossy(&chunk));
        loop {
            match buf.find('\n') {
                None => break,
                Some(pos) => {
                    let line = buf[..pos].trim().to_string();
                    buf = buf[pos + 1..].to_string();
                    if let Some(data) = line.strip_prefix("data:") {
                        on_data(data.trim())?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Helper: parse a newline-delimited JSON stream (Ollama).
async fn consume_ndjson<F>(
    response: reqwest::Response,
    mut on_line: F,
) -> Result<(), String>
where
    F: FnMut(&str) -> Result<(), String>,
{
    let mut stream = response.bytes_stream();
    let mut buf = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buf.push_str(&String::from_utf8_lossy(&chunk));
        loop {
            match buf.find('\n') {
                None => break,
                Some(pos) => {
                    let line = buf[..pos].trim().to_string();
                    buf = buf[pos + 1..].to_string();
                    if !line.is_empty() {
                        on_line(&line)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn run_prompt_streaming(
    channel: Channel<StreamEvent>,
    state: State<'_, AppState>,
    request: RunPromptRequest,
) -> Result<(), String> {
    let interpolated = interpolate_variables(&request.prompt, &request.variables);
    let prompt = prepend_text_attachments(&interpolated, &request.attachments);
    let images = image_attachments(&request.attachments);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let result: Result<(), String> = async {
        match request.provider.as_str() {
            "openai" => {
                let api_key = Keychain::get_api_key("openai").map_err(|e| e.to_string())?;
                let mut content_parts: Vec<Value> = vec![json!({"type":"text","text":prompt})];
                for img in &images {
                    content_parts.push(json!({
                        "type":"image_url",
                        "image_url":{"url":format!("data:{};base64,{}",img.mime_type,img.content)}
                    }));
                }
                let user_content: Value = if images.is_empty() { json!(prompt) } else { json!(content_parts) };
                let body = json!({
                    "model": request.model,
                    "messages": [{"role":"user","content":user_content}],
                    "temperature": request.temperature.unwrap_or(0.7),
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                    "stream": true
                });
                let response = client
                    .post("https://api.openai.com/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&body).send().await.map_err(|e| e.to_string())?;
                consume_sse(response, |data| {
                    if data == "[DONE]" { return Ok(()); }
                    if let Ok(v) = serde_json::from_str::<Value>(data) {
                        if let Some(text) = v["choices"][0]["delta"]["content"].as_str() {
                            if !text.is_empty() {
                                channel.send(StreamEvent::Token(text.to_string())).map_err(|e| e.to_string())?;
                            }
                        }
                    }
                    Ok(())
                }).await?;
            }
            "mistral" => {
                let api_key = Keychain::get_api_key("mistral").map_err(|e| e.to_string())?;
                let body = json!({
                    "model": request.model,
                    "messages": [{"role":"user","content":prompt}],
                    "temperature": request.temperature.unwrap_or(0.7),
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                    "stream": true
                });
                let response = client
                    .post("https://api.mistral.ai/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&body).send().await.map_err(|e| e.to_string())?;
                consume_sse(response, |data| {
                    if data == "[DONE]" { return Ok(()); }
                    if let Ok(v) = serde_json::from_str::<Value>(data) {
                        if let Some(text) = v["choices"][0]["delta"]["content"].as_str() {
                            if !text.is_empty() {
                                channel.send(StreamEvent::Token(text.to_string())).map_err(|e| e.to_string())?;
                            }
                        }
                    }
                    Ok(())
                }).await?;
            }
            "anthropic" => {
                let api_key = Keychain::get_api_key("anthropic").map_err(|e| e.to_string())?;
                let user_content: Value = if images.is_empty() {
                    json!(prompt)
                } else {
                    let mut parts: Vec<Value> = images.iter().map(|img| json!({
                        "type":"image","source":{"type":"base64","media_type":img.mime_type,"data":img.content}
                    })).collect();
                    parts.push(json!({"type":"text","text":prompt}));
                    json!(parts)
                };
                let body = json!({
                    "model": request.model,
                    "messages": [{"role":"user","content":user_content}],
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                    "stream": true
                });
                let response = client
                    .post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("Content-Type", "application/json")
                    .json(&body).send().await.map_err(|e| e.to_string())?;
                consume_sse(response, |data| {
                    if let Ok(v) = serde_json::from_str::<Value>(data) {
                        if v["type"] == "content_block_delta" {
                            if let Some(text) = v["delta"]["text"].as_str() {
                                if !text.is_empty() {
                                    channel.send(StreamEvent::Token(text.to_string())).map_err(|e| e.to_string())?;
                                }
                            }
                        }
                    }
                    Ok(())
                }).await?;
            }
            "ollama" => {
                let base_url = get_ollama_base_url(&state);
                let image_b64s: Vec<&str> = images.iter().map(|img| img.content.as_str()).collect();
                let mut body_map = json!({
                    "model": request.model,
                    "prompt": prompt,
                    "stream": true,
                    "options": {
                        "temperature": request.temperature.unwrap_or(0.7),
                        "num_predict": request.max_tokens.unwrap_or(4096)
                    }
                });
                if !image_b64s.is_empty() { body_map["images"] = json!(image_b64s); }
                let response = client
                    .post(format!("{}/api/generate", base_url))
                    .header("Content-Type", "application/json")
                    .json(&body_map).send().await
                    .map_err(|e| format!("Cannot reach Ollama at {}. Is it running? ({})", base_url, e))?;
                consume_ndjson(response, |line| {
                    if let Ok(v) = serde_json::from_str::<Value>(line) {
                        if let Some(token) = v["response"].as_str() {
                            if !token.is_empty() {
                                channel.send(StreamEvent::Token(token.to_string())).map_err(|e| e.to_string())?;
                            }
                        }
                    }
                    Ok(())
                }).await?;
            }
            "gemini" => {
                let api_key = Keychain::get_api_key("gemini").map_err(|e| e.to_string())?;
                let mut parts: Vec<Value> = vec![json!({"text": prompt})];
                for img in &images {
                    parts.push(json!({"inline_data":{"mime_type":img.mime_type,"data":img.content}}));
                }
                let body = json!({
                    "contents": [{"parts": parts}],
                    "generationConfig": {
                        "temperature": request.temperature.unwrap_or(0.7),
                        "maxOutputTokens": request.max_tokens.unwrap_or(4096)
                    }
                });
                let response = client
                    .post(format!(
                        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}&alt=sse",
                        request.model, api_key
                    ))
                    .header("Content-Type", "application/json")
                    .json(&body).send().await.map_err(|e| e.to_string())?;
                consume_sse(response, |data| {
                    if let Ok(v) = serde_json::from_str::<Value>(data) {
                        if let Some(text) = v["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                            if !text.is_empty() {
                                channel.send(StreamEvent::Token(text.to_string())).map_err(|e| e.to_string())?;
                            }
                        }
                    }
                    Ok(())
                }).await?;
            }
            other => return Err(format!("Unknown provider: {}", other)),
        }
        Ok(())
    }.await;

    match result {
        Ok(()) => { let _ = channel.send(StreamEvent::Done); Ok(()) }
        Err(e) => {
            let _ = channel.send(StreamEvent::Error(e.clone()));
            Err(e)
        }
    }
}

// ─── Multi-turn conversation ──────────────────────────────────────────────────

#[tauri::command]
pub async fn run_conversation(
    state: State<'_, AppState>,
    request: RunConversationRequest,
) -> Result<ProviderResponse, String> {
    let start = Instant::now();
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    match request.provider.as_str() {
        "openai" | "mistral" => {
            let (api_key, url) = if request.provider == "openai" {
                (Keychain::get_api_key("openai").map_err(|e| e.to_string())?,
                 "https://api.openai.com/v1/chat/completions".to_string())
            } else {
                (Keychain::get_api_key("mistral").map_err(|e| e.to_string())?,
                 "https://api.mistral.ai/v1/chat/completions".to_string())
            };
            let messages: Vec<Value> = request.messages.iter()
                .map(|m| json!({"role": m.role, "content": m.content}))
                .collect();
            let body = json!({
                "model": request.model,
                "messages": messages,
                "temperature": request.temperature.unwrap_or(0.7),
                "max_tokens": request.max_tokens.unwrap_or(4096)
            });
            let response = client.post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body).send().await.map_err(|e| e.to_string())?;
            let data: Value = response.json().await.map_err(|e| e.to_string())?;
            let content = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
            let input_tokens = data["usage"]["prompt_tokens"].as_u64().map(|v| v as u32);
            let output_tokens = data["usage"]["completion_tokens"].as_u64().map(|v| v as u32);
            Ok(ProviderResponse { content, model: request.model, tokens_input: input_tokens, tokens_output: output_tokens, duration_ms: Some(start.elapsed().as_millis() as u64) })
        }
        "anthropic" => {
            let api_key = Keychain::get_api_key("anthropic").map_err(|e| e.to_string())?;
            let system_content = request.messages.iter().find(|m| m.role == "system").map(|m| m.content.clone());
            let non_system: Vec<Value> = request.messages.iter()
                .filter(|m| m.role != "system")
                .map(|m| json!({"role": m.role, "content": m.content}))
                .collect();
            let mut body = json!({
                "model": request.model,
                "messages": non_system,
                "max_tokens": request.max_tokens.unwrap_or(4096)
            });
            if let Some(sys) = system_content { body["system"] = json!(sys); }
            let response = client.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body).send().await.map_err(|e| e.to_string())?;
            let data: Value = response.json().await.map_err(|e| e.to_string())?;
            let content = data["content"][0]["text"].as_str().unwrap_or("").to_string();
            let input_tokens = data["usage"]["input_tokens"].as_u64().map(|v| v as u32);
            let output_tokens = data["usage"]["output_tokens"].as_u64().map(|v| v as u32);
            Ok(ProviderResponse { content, model: request.model, tokens_input: input_tokens, tokens_output: output_tokens, duration_ms: Some(start.elapsed().as_millis() as u64) })
        }
        "ollama" => {
            let base_url = get_ollama_base_url(&state);
            let messages: Vec<Value> = request.messages.iter()
                .map(|m| json!({"role": m.role, "content": m.content}))
                .collect();
            let body = json!({
                "model": request.model,
                "messages": messages,
                "stream": false
            });
            let response = client.post(format!("{}/api/chat", base_url))
                .header("Content-Type", "application/json")
                .json(&body).send().await
                .map_err(|e| format!("Cannot reach Ollama at {}. Is it running? ({})", base_url, e))?;
            let data: Value = response.json().await.map_err(|e| e.to_string())?;
            let content = data["message"]["content"].as_str().unwrap_or("").to_string();
            Ok(ProviderResponse { content, model: request.model, tokens_input: None, tokens_output: None, duration_ms: Some(start.elapsed().as_millis() as u64) })
        }
        "gemini" => {
            let api_key = Keychain::get_api_key("gemini").map_err(|e| e.to_string())?;
            let system_content = request.messages.iter().find(|m| m.role == "system").map(|m| m.content.clone());
            let contents: Vec<Value> = request.messages.iter()
                .filter(|m| m.role != "system")
                .map(|m| json!({
                    "role": if m.role == "assistant" { "model" } else { "user" },
                    "parts": [{"text": m.content}]
                }))
                .collect();
            let mut body = json!({
                "contents": contents,
                "generationConfig": {
                    "temperature": request.temperature.unwrap_or(0.7),
                    "maxOutputTokens": request.max_tokens.unwrap_or(4096)
                }
            });
            if let Some(sys) = system_content {
                body["system_instruction"] = json!({"parts": [{"text": sys}]});
            }
            let response = client
                .post(format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                    request.model, api_key
                ))
                .header("Content-Type", "application/json")
                .json(&body).send().await.map_err(|e| e.to_string())?;
            let data: Value = response.json().await.map_err(|e| e.to_string())?;
            let content = data["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("").to_string();
            Ok(ProviderResponse { content, model: request.model, tokens_input: None, tokens_output: None, duration_ms: Some(start.elapsed().as_millis() as u64) })
        }
        other => Err(format!("Unknown provider: {}", other)),
    }
}

#[tauri::command]
pub async fn stream_prompt(state: State<'_, AppState>, request: StreamPromptRequest) -> Result<String, String> {
    // For simplicity, we'll just return the non-streaming response
    // Full streaming would require WebSocket or SSE setup
    let non_streaming_request = RunPromptRequest {
        provider: request.provider,
        model: request.model,
        prompt: request.prompt,
        variables: request.variables,
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        system_prompt: request.system_prompt,
        attachments: request.attachments,
    };
    
    let response = run_prompt(state, non_streaming_request).await?;
    Ok(response.content)
}

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>, provider: String) -> Result<Vec<ModelInfo>, String> {
    match provider.as_str() {
        "openai" => {
            let api_key = Keychain::get_api_key("openai").map_err(|e| e.to_string())?;
            let client = Client::new();
            
            let response = client
                .get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            
            let models: Vec<ModelInfo> = data["data"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter(|m| m["id"].as_str().map(|s| s.starts_with("gpt-")).unwrap_or(false))
                .map(|m| ModelInfo {
                    id: m["id"].as_str().unwrap_or("").to_string(),
                    name: m["id"].as_str().unwrap_or("").to_string(),
                    context_window: m["context_window"].as_u64().map(|v| v as u32),
                })
                .collect();
            
            Ok(models)
        }
        "anthropic" => {
            // Anthropic doesn't have a public models list API, return known models
            Ok(vec![
                ModelInfo { id: "claude-opus-4-6".to_string(), name: "Claude Opus 4.6".to_string(), context_window: Some(200000) },
                ModelInfo { id: "claude-sonnet-4-6".to_string(), name: "Claude Sonnet 4.6".to_string(), context_window: Some(200000) },
                ModelInfo { id: "claude-haiku-4-5-20251001".to_string(), name: "Claude Haiku 4.5".to_string(), context_window: Some(200000) },
            ])
        }
        "ollama" => {
            let base_url = get_ollama_base_url(&state);
            let client = Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|e| e.to_string())?;

            let response = client
                .get(format!("{}/api/tags", base_url))
                .send()
                .await
                .map_err(|e| format!("Cannot reach Ollama at {}. Is it running? ({})", base_url, e))?;

            let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

            let models: Vec<ModelInfo> = data["models"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|m| ModelInfo {
                    id: m["name"].as_str().unwrap_or("").to_string(),
                    name: m["name"].as_str().unwrap_or("").to_string(),
                    context_window: None,
                })
                .collect();

            Ok(models)
        }
        "gemini" => {
            // Return known Gemini models
            Ok(vec![
                ModelInfo { id: "gemini-2.0-flash".to_string(), name: "Gemini 2.0 Flash".to_string(), context_window: Some(1048576) },
                ModelInfo { id: "gemini-2.0-pro".to_string(), name: "Gemini 2.0 Pro".to_string(), context_window: Some(2097152) },
                ModelInfo { id: "gemini-1.5-pro".to_string(), name: "Gemini 1.5 Pro".to_string(), context_window: Some(2097152) },
                ModelInfo { id: "gemini-1.5-flash".to_string(), name: "Gemini 1.5 Flash".to_string(), context_window: Some(1048576) },
            ])
        }
        "mistral" => Ok(vec![
            ModelInfo { id: "mistral-large-latest".to_string(), name: "Mistral Large".to_string(), context_window: Some(128000) },
            ModelInfo { id: "mistral-small-latest".to_string(), name: "Mistral Small".to_string(), context_window: Some(128000) },
            ModelInfo { id: "open-mistral-nemo".to_string(), name: "Mistral Nemo".to_string(), context_window: Some(128000) },
            ModelInfo { id: "codestral-latest".to_string(), name: "Codestral".to_string(), context_window: Some(256000) },
        ]),
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

#[tauri::command]
pub async fn health_check(state: State<'_, AppState>, provider: String) -> Result<HealthStatus, String> {
    match provider.as_str() {
        "openai" => {
            match Keychain::get_api_key("openai") {
                Ok(api_key) => {
                    let client = Client::builder()
                        .timeout(std::time::Duration::from_secs(10))
                        .build()
                        .map_err(|e| e.to_string())?;
                    match client.get("https://api.openai.com/v1/models")
                        .header("Authorization", format!("Bearer {}", api_key))
                        .send()
                        .await {
                        Ok(resp) if resp.status().is_success() => Ok(HealthStatus { provider: "openai".to_string(), available: true, error: None }),
                        Ok(resp) => Ok(HealthStatus { provider: "openai".to_string(), available: false, error: Some(resp.status().to_string()) }),
                        Err(e) => Ok(HealthStatus { provider: "openai".to_string(), available: false, error: Some(e.to_string()) }),
                    }
                }
                Err(_) => Ok(HealthStatus { provider: "openai".to_string(), available: false, error: Some("API key not configured".to_string()) }),
            }
        }
        "anthropic" => {
            match Keychain::get_api_key("anthropic") {
                Ok(_) => Ok(HealthStatus { provider: "anthropic".to_string(), available: true, error: None }),
                Err(_) => Ok(HealthStatus { provider: "anthropic".to_string(), available: false, error: Some("API key not configured".to_string()) }),
            }
        }
        "ollama" => {
            let base_url = get_ollama_base_url(&state);
            let client = Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default();
            match client.get(format!("{}/api/tags", base_url)).send().await {
                Ok(resp) if resp.status().is_success() => Ok(HealthStatus { provider: "ollama".to_string(), available: true, error: None }),
                Ok(resp) => Ok(HealthStatus { provider: "ollama".to_string(), available: false, error: Some(resp.status().to_string()) }),
                Err(_) => Ok(HealthStatus { provider: "ollama".to_string(), available: false, error: Some(format!("Cannot reach Ollama at {}. Is it running?", base_url)) }),
            }
        }
        "gemini" => {
            match Keychain::get_api_key("gemini") {
                Ok(_) => Ok(HealthStatus { provider: "gemini".to_string(), available: true, error: None }),
                Err(_) => Ok(HealthStatus { provider: "gemini".to_string(), available: false, error: Some("API key not configured".to_string()) }),
            }
        }
        "mistral" => {
            match Keychain::get_api_key("mistral") {
                Ok(_) => Ok(HealthStatus { provider: "mistral".to_string(), available: true, error: None }),
                Err(_) => Ok(HealthStatus { provider: "mistral".to_string(), available: false, error: Some("API key not configured".to_string()) }),
            }
        }
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

#[tauri::command]
pub async fn store_api_key(provider: String, api_key: String) -> Result<(), String> {
    Keychain::store_api_key(&provider, &api_key).map_err(|e| e.to_string())?;
    info!("Stored API key for provider: {}", provider);
    Ok(())
}

#[tauri::command]
pub async fn get_api_key_status(provider: String) -> Result<ApiKeyStatus, String> {
    let has_key = Keychain::has_api_key(&provider);
    Ok(ApiKeyStatus {
        provider,
        configured: has_key,
        has_key,
    })
}