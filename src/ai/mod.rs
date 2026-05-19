mod error;
mod raw;

pub use error::AiError;

use serde::{Deserialize, Serialize};

// ── 请求类型 ──

#[derive(Serialize, Debug, Clone)]
pub struct CreateMessageRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
    Blocks(Vec<SystemTextBlock>),
}

#[derive(Serialize, Debug, Clone)]
pub struct SystemTextBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Image {
        source: ImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: ToolResultContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ToolResultContentBlock>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolResultContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: String,
}

// ── 响应类型 ──

#[derive(Deserialize, Debug, Clone)]
pub struct MessageResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub model: String,
    pub content: Vec<ResponseContentBlock>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    Thinking {
        thinking: String,
        #[serde(default)]
        signature: String,
    },
    RedactedThinking {
        data: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u32,
    pub output_tokens: u32,
}

// ── 流式事件类型 ──

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart {
        message: StreamMessageStart,
    },
    ContentBlockStart {
        index: u32,
        content_block: ResponseContentBlock,
    },
    ContentBlockDelta {
        index: u32,
        delta: ContentBlockDeltaValue,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageDelta {
        delta: MessageDeltaValue,
        usage: Option<Usage>,
    },
    MessageStop,
    Ping,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StreamMessageStart {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub model: String,
    pub content: Vec<ResponseContentBlock>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockDeltaValue {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    ThinkingDelta { thinking: String },
    SignatureDelta { signature: String },
}

#[derive(Deserialize, Debug, Clone)]
pub struct MessageDeltaValue {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

// ── 公开 API ──

/// 创建非流式消息
///
/// 向 AI 接口发送请求，等待完整响应后返回。
pub async fn create_message(
    api_key: &str,
    request: CreateMessageRequest,
) -> Result<MessageResponse, crate::Error<AiError>> {
    raw::raw_create_message(api_key, request).await
}

/// 创建流式消息
///
/// 返回 [`tokio::sync::mpsc::Receiver`]，逐事件接收 SSE 流。
/// 调用方 drop receiver 即可取消。
pub async fn create_message_stream(
    api_key: &str,
    request: CreateMessageRequest,
) -> Result<
    tokio::sync::mpsc::Receiver<Result<StreamEvent, crate::Error<AiError>>>,
    crate::Error<AiError>,
> {
    raw::raw_create_message_stream(api_key, request).await
}

// ── 测试 ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{TEST_AI_API_KEY, TEST_AI_MODEL};

    fn make_request(content: &str) -> CreateMessageRequest {
        CreateMessageRequest {
            model: TEST_AI_MODEL.into(),
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text(content.into()),
            }],
            max_tokens: 100,
            system: None,
            tools: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: None,
            metadata: None,
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_message() {
        let request = make_request("Hello, Claude.");
        let response = create_message(TEST_AI_API_KEY, request).await.unwrap();
        assert_eq!(response.response_type, "message");
        assert!(!response.content.is_empty());
        assert!(response.usage.output_tokens > 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_message_stream() {
        let request = make_request("Hi");
        let mut rx = create_message_stream(TEST_AI_API_KEY, request)
            .await
            .unwrap();
        let mut event_count = 0;
        while let Some(event) = rx.recv().await {
            let _ = event.unwrap();
            event_count += 1;
        }
        assert!(event_count > 0);
    }
}
