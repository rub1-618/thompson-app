use crate::core::{HistoryEntry, Roles, SettingsEntry};
use serde::{Deserialize, Serialize};

const OLLAMA_URL: &str = "http://localhost:11434/api/chat";
const SYSTEM_PROMPT: &str = "Відповідай стисло і без емодзі.";
const CONTEXT_TURNS: usize = 2; // the amount of last replicas as context
const ALLOW_STREAM: bool = false;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    keep_alive: i32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

fn role_name(role: Roles) -> String {
    match role {
        Roles::Tom => "assistant".to_string(),
        Roles::User => "user".to_string(),
    }
}

pub async fn ask_ai(prompt: &str, history: &[HistoryEntry], settings: &SettingsEntry) -> String {
    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    }];

    let tail = &history[history.len().saturating_sub(CONTEXT_TURNS)..];
    for entry in tail {
        messages.push(ChatMessage {
            role: role_name(entry.role),
            content: entry.text.clone() 
        });
    }
    
    messages.push(ChatMessage { 
        role: "user".to_string(),
        content: prompt.to_string()
    });

    let request = ChatRequest {
        model: settings.local_model.to_string(),
        messages,
        stream: ALLOW_STREAM,
        keep_alive: -1,
    };

    let response = match reqwest::Client::new()
        .post(OLLAMA_URL)
        .json(&request)
        .send()
        .await {
            Ok(r) => r,
            Err(e) => if e.is_connect() {
                return "Ollama недоступна. Перевір що сервіс запущений.".to_string();
            } else {
                return "Ollama: помилка запиту.".to_string()
            }
        };

    let parsed: ChatResponse = match response.json().await {
        Ok(p) => p,
        Err(_) => return "Ollama: помилка запиту.".to_string(),
    };

    let text = parsed.message.content.trim().to_string();
    if text.is_empty() {
        "Не вдалось отримати відповідь.".to_string()
    } else { text }
}
