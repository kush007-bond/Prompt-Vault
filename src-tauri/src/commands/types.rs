use serde::{Deserialize, Serialize};

// ==================== Prompt Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub title: String,
    pub body: String,
    pub model_target: Option<String>,
    pub collection_id: Option<String>,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub use_count: i32,
    pub sort_order: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePromptRequest {
    pub title: String,
    pub body: String,
    pub model_target: Option<String>,
    pub collection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePromptRequest {
    pub id: String,
    pub title: String,
    pub body: String,
    pub model_target: Option<String>,
    pub collection_id: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub collection_id: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersion {
    pub id: String,
    pub prompt_id: String,
    pub title_snapshot: String,
    pub body_snapshot: String,
    pub changed_at: String,
    pub change_note: Option<String>,
}

// ==================== Collection Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_smart: bool,
    pub smart_filter: Option<String>,
    pub sort_order: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCollectionRequest {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

// ==================== Tag Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTagRequest {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

// ==================== Settings Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSettingRequest {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSettingRequest {
    pub key: String,
    pub value: String,
}

// ==================== AI Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// File name (e.g. "report.txt", "photo.png")
    pub name: String,
    /// Plain UTF-8 text for text files; raw base64 (no data-URL prefix) for images
    pub content: String,
    /// MIME type, e.g. "text/plain", "image/jpeg"
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunPromptRequest {
    pub provider: String,
    pub model: String,
    pub prompt: String,
    pub variables: Option<std::collections::HashMap<String, String>>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamPromptRequest {
    pub provider: String,
    pub model: String,
    pub prompt: String,
    pub variables: Option<std::collections::HashMap<String, String>>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub content: String,
    pub model: String,
    pub tokens_input: Option<u32>,
    pub tokens_output: Option<u32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub provider: String,
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStatus {
    pub provider: String,
    pub configured: bool,
    pub has_key: bool,
}

// ==================== Conversation Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConversationRequest {
    pub provider: String,
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Streamed over a Tauri Channel for real-time AI output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum StreamEvent {
    #[serde(rename = "token")]
    Token(String),
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "error")]
    Error(String),
}

// ==================== CLI Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectRequest {
    pub prompt_id: String,
    pub content: String,
    pub project_path: String,
    pub apply_globally: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionResult {
    pub success: bool,
    pub target_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionPreview {
    pub target_path: String,
    pub content_preview: String,
    pub will_overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionLog {
    pub id: String,
    pub prompt_id: Option<String>,
    pub cli_target: String,
    pub project_path: String,
    pub injected_content: String,
    pub injected_at: String,
}

// ==================== Export/Import Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: String,
    pub prompts: Vec<ExportedPrompt>,
    pub collections: Vec<ExportedCollection>,
    pub tags: Vec<ExportedTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedPrompt {
    pub id: String,
    pub title: String,
    pub body: String,
    pub model_target: Option<String>,
    pub collection_id: Option<String>,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub use_count: i32,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedCollection {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedTag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub json_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub prompts_imported: i32,
    pub collections_imported: i32,
    pub tags_imported: i32,
    pub message: String,
}