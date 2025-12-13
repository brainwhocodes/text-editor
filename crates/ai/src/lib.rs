use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("missing OpenRouter API key")]
    MissingApiKey,

    #[error("keyring error: {0}")]
    Keyring(String),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct KeyStore {
    service: String,
    username: String,
}

impl KeyStore {
    pub fn new(service: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            username: username.into(),
        }
    }

    pub fn set_openrouter_key(&self, key: &str) -> Result<(), AiError> {
        let entry = keyring::Entry::new(&self.service, &self.username)
            .map_err(|e| AiError::Keyring(e.to_string()))?;
        entry
            .set_password(key)
            .map_err(|e| AiError::Keyring(e.to_string()))
    }

    pub fn get_openrouter_key(&self) -> Result<Option<String>, AiError> {
        let entry = keyring::Entry::new(&self.service, &self.username)
            .map_err(|e| AiError::Keyring(e.to_string()))?;
        match entry.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AiError::Keyring(e.to_string())),
        }
    }

    pub fn remove_openrouter_key(&self) -> Result<(), AiError> {
        let entry = keyring::Entry::new(&self.service, &self.username)
            .map_err(|e| AiError::Keyring(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AiError::Keyring(e.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenRouterClient {
    http: reqwest::Client,
}

impl OpenRouterClient {
    pub fn new() -> Result<Self, AiError> {
        Ok(Self {
            http: reqwest::Client::new(),
        })
    }

    pub async fn chat_completions(
        &self,
        api_key: &str,
        request: ChatCompletionsRequest,
    ) -> Result<ChatCompletionsResponse, AiError> {
        let resp = self
            .http
            .post("https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.json::<ChatCompletionsResponse>().await?)
    }

    pub async fn chat_completions_stream(
        &self,
        api_key: &str,
        mut request: ChatCompletionsRequest,
        buffer: usize,
    ) -> Result<mpsc::Receiver<Result<String, AiError>>, AiError> {
        request.stream = Some(true);

        let resp = self
            .http
            .post("https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let (tx, rx) = mpsc::channel(buffer);

        tokio::spawn(async move {
            let mut stream = resp.bytes_stream();
            let mut buf: Vec<u8> = Vec::new();

            while let Some(item) = futures_util::StreamExt::next(&mut stream).await {
                match item {
                    Ok(chunk) => {
                        buf.extend_from_slice(&chunk);

                        while let Some((event, rest)) = split_sse_event(&buf) {
                            buf = rest;

                            let data = match sse_extract_data(event.as_slice()) {
                                Ok(v) => v,
                                Err(e) => {
                                    let _ = tx.send(Err(e)).await;
                                    return;
                                }
                            };

                            if data == "[DONE]" {
                                return;
                            }

                            match serde_json::from_str::<ChatCompletionsStreamResponse>(&data) {
                                Ok(r) => {
                                    for choice in r.choices {
                                        if let Some(delta) = choice.delta.and_then(|d| d.content) {
                                            if !delta.is_empty() {
                                                let _ = tx.send(Ok(delta)).await;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(AiError::Json(e))).await;
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(AiError::Http(e))).await;
                        return;
                    }
                }
            }
        });

        Ok(rx)
    }
}

#[derive(Debug, Clone)]
pub struct AiService {
    client: OpenRouterClient,
    key_store: KeyStore,
}

impl AiService {
    pub fn new(client: OpenRouterClient, key_store: KeyStore) -> Self {
        Self { client, key_store }
    }

    pub fn key_store(&self) -> &KeyStore {
        &self.key_store
    }

    pub async fn send_chat(
        &self,
        request: ChatCompletionsRequest,
    ) -> Result<ChatCompletionsResponse, AiError> {
        let key = self
            .key_store
            .get_openrouter_key()?
            .ok_or(AiError::MissingApiKey)?;
        self.client.chat_completions(&key, request).await
    }

    pub async fn send_chat_stream(
        &self,
        request: ChatCompletionsRequest,
        buffer: usize,
    ) -> Result<mpsc::Receiver<Result<String, AiError>>, AiError> {
        let key = self
            .key_store
            .get_openrouter_key()?
            .ok_or(AiError::MissingApiKey)?;
        self.client
            .chat_completions_stream(&key, request, buffer)
            .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionsRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionsResponse {
    pub id: String,
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionsStreamResponse {
    pub id: Option<String>,
    pub choices: Vec<ChatStreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChoice {
    pub index: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<ChatStreamDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

fn split_sse_event(buf: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut i = 0;
    while i < buf.len() {
        if i + 1 < buf.len() && buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some((buf[..i].to_vec(), buf[(i + 2)..].to_vec()));
        }

        if i + 3 < buf.len()
            && buf[i] == b'\r'
            && buf[i + 1] == b'\n'
            && buf[i + 2] == b'\r'
            && buf[i + 3] == b'\n'
        {
            return Some((buf[..i].to_vec(), buf[(i + 4)..].to_vec()));
        }

        i += 1;
    }

    None
}

fn sse_extract_data(event: &[u8]) -> Result<String, AiError> {
    let text = String::from_utf8_lossy(event);
    let mut data_lines: Vec<&str> = Vec::new();

    for line in text.lines() {
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start());
        }
    }

    Ok(data_lines.join("\n"))
}
