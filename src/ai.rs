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
    match settings.ai_mode.as_str() {
        "gemini" => ask_gemini(prompt, history, settings).await,
        _ => ask_ollama(prompt, history, settings).await,
    }
}

pub async fn ask_ollama(prompt: &str, history: &[HistoryEntry], settings: &SettingsEntry) -> String {
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

// ! cloud gemini ai

#[derive(Serialize)]
struct GeminiRequest {
    system_instruction: GeminiSystem,
    contents: Vec<GeminiContent>,
}
#[derive(Serialize)]
struct GeminiSystem { parts: Vec<GeminiPart> }
#[derive(Serialize)]
struct GeminiContent { role: String, parts: Vec<GeminiPart> }
#[derive(Serialize, Deserialize)]
struct GeminiPart { text: String }

#[derive(Deserialize)]
struct GeminiResponse { candidates: Vec<GeminiCandidate> }
#[derive(Deserialize)]
struct GeminiCandidate { content: GeminiContentResp }
#[derive(Deserialize)]
struct GeminiContentResp { parts: Vec<GeminiPart> }

fn gemini_role(role: Roles) -> String {
    match role {
        Roles::Tom => "model".to_string(),
        Roles::User => "user".to_string()
    }
}

pub async fn ask_gemini(prompt: &str, history: &[HistoryEntry], settings: &SettingsEntry) -> String {
    if settings.gemini_key.is_empty() {
        return "API-ключ для Gemini не знайдено.".to_string()
    }
    let model = settings.map.get("gemini_model")
        .and_then(|v| v.as_str()).unwrap_or("gemini-2.0-flash");

    let mut  contents: Vec<GeminiContent> = Vec::new();
    let tail = &history[history.len().saturating_sub(CONTEXT_TURNS)..];
    for entry in tail {
        contents.push(GeminiContent { 
            role: gemini_role(entry.role),
            parts: vec![GeminiPart { text: entry.text.clone() }]
        });
    }
    contents.push(GeminiContent { 
        role: "user".to_string(), 
        parts: vec![GeminiPart { text: prompt.to_string() }]
    });

    let request = GeminiRequest {
        system_instruction: GeminiSystem { parts: vec![GeminiPart { text: SYSTEM_PROMPT.to_string() }]},
        contents,
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={}",
        settings.gemini_key
    );

    let resp = match reqwest::Client::new().post(&url).json(&request).send().await {
        Ok(r) => r,
        Err(_) => return "Gemini: сервери переповнені.".to_string(),
    };

    match resp.status().as_u16() {
        200 => {}
        429 => return "Gemini: перевищено квоту, спробуйте пізніше.".to_string(),
        400 | 403 => return "Ви на авторизованi або невказаний ключ.".to_string(),
        _ => return "Gemini: сервери переповнені.".to_string(),
    }

    let parsed: GeminiResponse = match resp.json().await {
        Ok(p) => p,
        Err(_) => return "Gemini: сервери переповнені.".to_string(),
    };

    let text = parsed.candidates.first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.trim().to_string())
        .unwrap_or_default();
    if text.is_empty() {
        "Не вдалось отримати відповідь.".to_string()
    } else { text }
}

// ! cloud gemini-vision ai

pub async fn ask_gemini_vision(image_b64: &str, prompt: &str, settings: &SettingsEntry) -> String {
    if settings.gemini_key.is_empty() {
        return "API-ключ для Gemini не знайдено.".to_string()
    }
    let model = settings.map.get("gemini_model")
        .and_then(|v| v.as_str()).unwrap_or("gemini-2.0-flash");

    let body = serde_json::json!({
        "contents": [{
            "parts": [
                { "text": &prompt.to_string() },
                { "inline_data" : { "mime_type": "image/png", "data": image_b64 } }
            ]
        }]
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={}",
        settings.gemini_key
    );

    let resp = match::reqwest::Client::new().post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(_) => return "Gemini: сервери переповнені.".to_string(),
    };
    match resp.status().as_u16() {
        200 => {}
        429 => return "Gemini: перевищено квоту, спробуйте пізніше.".to_string(),
        400 | 403 => return "Ви на авторизованi або невказаний ключ.".to_string(),
        _ => return "Gemini: сервери переповнені.".to_string(),
    };

    let parsed: GeminiResponse = match resp.json().await {
        Ok(p) => p,
        Err(_) => return "Gemini: сервери переповнені.".to_string(),
    };
    let text = parsed.candidates.first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.trim().to_string())
        .unwrap_or_default();

    if text.is_empty() {
        "Не вдалось отримати відповідь.".to_string()
    } else { text }
}