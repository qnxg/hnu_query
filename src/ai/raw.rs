use crate::{
    ai::error::AiError,
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr},
    utils::client,
};

use super::{CreateMessageRequest, MessageResponse, StreamEvent};

const AI_BASE_URL: &str = "https://maas.nscc-cs.cn/external/api";

// Anthropic Messages API （还没验证是否都支持）
// POST /v1/messages
// POST /v1/messages/count_tokens
// GET  /v1/models
// GET  /v1/models/{model_id}

#[derive(serde::Deserialize)]
struct ApiErrorBody {
    error: ApiErrorDetail,
}

#[derive(serde::Deserialize)]
struct ApiErrorDetail {
    #[serde(rename = "type")]
    #[expect(dead_code)]
    error_type: String,
    message: String,
}

async fn send_request(
    api_key: &str,
    request: &CreateMessageRequest,
) -> Result<reqwest::Response, crate::Error<AiError>> {
    let response = client
        .post(format!("{AI_BASE_URL}/v1/messages"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(request)
        .send()
        .await
        .network_err()?;

    let status = response.status();
    if !status.is_success() {
        let body_text = response.text().await.unexpected_err()?;
        return Err(parse_api_error(status.as_u16(), &body_text));
    }

    Ok(response)
}

pub(super) async fn raw_create_message(
    api_key: &str,
    mut request: CreateMessageRequest,
) -> Result<MessageResponse, crate::Error<AiError>> {
    request.stream = Some(false);
    let response = send_request(api_key, &request).await?;
    let body_text = response.text().await.unexpected_err()?;
    serde_json::from_str::<MessageResponse>(&body_text)
        .parse_err_with_reason(&body_text, "failed to parse message response")
}

pub(super) async fn raw_create_message_stream(
    api_key: &str,
    mut request: CreateMessageRequest,
) -> Result<
    tokio::sync::mpsc::Receiver<Result<StreamEvent, crate::Error<AiError>>>,
    crate::Error<AiError>,
> {
    request.stream = Some(true);
    let response = send_request(api_key, &request).await?;

    let (tx, rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move {
        let result = parse_sse_stream(response, &tx).await;
        if let Err(e) = result {
            let _ = tx.send(Err(e)).await;
        }
    });

    Ok(rx)
}

fn parse_api_error(status: u16, body: &str) -> crate::Error<AiError> {
    if let Ok(err_body) = serde_json::from_str::<ApiErrorBody>(body) {
        let ai_error = match status {
            401 => AiError::Authentication(err_body.error.message),
            429 => AiError::RateLimit(err_body.error.message),
            400 => AiError::BadRequest(err_body.error.message),
            402 => AiError::InsufficientQuota(err_body.error.message),
            422 => AiError::InvalidParameter(err_body.error.message),
            500 => AiError::ServerError(err_body.error.message),
            503 => AiError::ServiceUnavailable(err_body.error.message),
            _ => AiError::ApiError {
                status,
                message: err_body.error.message,
            },
        };
        crate::Error::Other(ai_error)
    } else {
        crate::Error::Other(AiError::ApiError {
            status,
            message: body.to_string(),
        })
    }
}

async fn parse_sse_stream(
    mut response: reqwest::Response,
    tx: &tokio::sync::mpsc::Sender<Result<StreamEvent, crate::Error<AiError>>>,
) -> Result<(), crate::Error<AiError>> {
    let mut buffer: Vec<u8> = Vec::new();

    loop {
        let chunk = response.chunk().await.network_err()?;
        match chunk {
            Some(bytes) => buffer.extend_from_slice(&bytes),
            None => break,
        }

        while let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
            let event_bytes: Vec<u8> = buffer.drain(..pos + 2).collect();
            let event_str = std::str::from_utf8(
                &event_bytes[..event_bytes.len().saturating_sub(2)],
            )
            .map_err(|e| {
                crate::Error::Other(AiError::StreamParse(format!(
                    "invalid UTF-8 in SSE event: {e}"
                )))
            })?;

            if let Some(data) = event_str
                .lines()
                .find_map(|line| line.strip_prefix("data: "))
            {
                match serde_json::from_str::<StreamEvent>(data) {
                    Ok(event) => {
                        if tx.send(Ok(event)).await.is_err() {
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(crate::Error::Other(AiError::StreamParse(format!(
                                "failed to parse SSE data: {e} | raw: {data}"
                            )))))
                            .await;
                    }
                }
            }
        }
    }

    Ok(())
}
