use crate::AppState;
use crate::commands::types::*;
use crate::keychain::Keychain;
use tauri::State;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;
use log::info;

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

#[tauri::command]
pub async fn run_prompt(state: State<'_, AppState>, request: RunPromptRequest) -> Result<ProviderResponse, String> {
    let start = Instant::now();
    let prompt = interpolate_variables(&request.prompt, &request.variables);
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    match request.provider.as_str() {
        "openai" => {
            let api_key = Keychain::get_api_key("openai").map_err(|e| e.to_string())?;
            
            let body = json!({
                "model": request.model,
                "messages": [
                    {"role": "user", "content": prompt}
                ],
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
            
            let body = json!({
                "model": request.model,
                "messages": [
                    {"role": "user", "content": prompt}
                ],
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
            let body = json!({
                "model": request.model,
                "prompt": prompt,
                "stream": false,
                "options": {
                    "temperature": request.temperature.unwrap_or(0.7),
                    "num_predict": request.max_tokens.unwrap_or(4096)
                }
            });
            
            let response = client
                .post("http://localhost:11434/api/generate")
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
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
            
            let body = json!({
                "contents": [{
                    "parts": [{"text": prompt}]
                }],
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
        _ => Err(format!("Unknown provider: {}", request.provider)),
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
    };
    
    let response = run_prompt(state, non_streaming_request).await?;
    Ok(response.content)
}

#[tauri::command]
pub async fn list_models(provider: String) -> Result<Vec<ModelInfo>, String> {
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
            let client = Client::new();
            
            let response = client
                .get("http://localhost:11434/api/tags")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
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
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

#[tauri::command]
pub async fn health_check(provider: String) -> Result<HealthStatus, String> {
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
            let client = Client::new();
            match client.get("http://localhost:11434/api/tags").send().await {
                Ok(resp) if resp.status().is_success() => Ok(HealthStatus { provider: "ollama".to_string(), available: true, error: None }),
                Ok(resp) => Ok(HealthStatus { provider: "ollama".to_string(), available: false, error: Some(resp.status().to_string()) }),
                Err(e) => Ok(HealthStatus { provider: "ollama".to_string(), available: false, error: Some(e.to_string()) }),
            }
        }
        "gemini" => {
            match Keychain::get_api_key("gemini") {
                Ok(_) => Ok(HealthStatus { provider: "gemini".to_string(), available: true, error: None }),
                Err(_) => Ok(HealthStatus { provider: "gemini".to_string(), available: false, error: Some("API key not configured".to_string()) }),
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