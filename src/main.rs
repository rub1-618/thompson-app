#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{core::{CommandEntry, HistoryEntry, SettingsEntry}, dispatcher::Dispatcher};
use std::sync::{atomic::Ordering, mpsc::Sender};
use core::MemoryStore;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex;

mod ai;
mod auto;
mod commands;
mod core;
mod dispatcher;
mod tts;
mod voice;

struct AppState {
    dispatcher: Dispatcher,
    store: MemoryStore,
    tts: Sender<tts::TtsCommand>
}

struct VoiceControl {
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    events: tokio::sync::mpsc::UnboundedSender<voice::VoiceEvent>,
    tts_speaking: std::sync::Arc::<std::sync::atomic::AtomicBool>,
    model_path: String,
}

#[tauri::command]
async fn execute_command(
    text: String, 
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let mut guard = state.lock().await;
    let AppState { dispatcher, store, tts } = &mut *guard;
    if let Some((crate::commands::Command::Stop,_)) = crate::dispatcher::find_command(&text) {
        let _ = tts.send(tts::TtsCommand::Stop);
    }
    let response = dispatcher.execute(&text, store).await;
    if !response.is_empty() {
        let _ = tts.send(tts::TtsCommand::Speak(response.clone()));
    }
    Ok(response)
}

#[tauri::command]
async fn toggle_ai_mode(state: State<'_, Mutex<AppState>>) -> Result<String, String> {
    let mut guard = state.lock().await;
    let next = if guard.store.settings.ai_mode == "gemini" { "ollama" } else { "gemini" };
    guard.store.settings.ai_mode = next.to_string();
    guard.store.save_settings().await.map_err(|e| e.to_string())?;
    Ok(next.to_string())
}

#[tauri::command]
async fn toggle_mute(
    state: State<'_, Mutex<AppState>>,
) -> Result<bool, String> {
    let mut guard = state.lock().await;
    guard.store.settings.mute_status = !guard.store.settings.mute_status;
    guard.store.save_settings().await.map_err(|e| e.to_string())?;
    let muted = guard.store.settings.mute_status;
    let _ = guard.tts.send(tts::TtsCommand::SetMute(muted));
    Ok(muted)
}

#[tauri::command]
fn toggle_listening(control: State<'_, VoiceControl>) -> Result<bool, String> {
    let status = !control.running.load(Ordering::Relaxed);
    control.running.store(status, Ordering::Relaxed);
    if status {
        let running = control.running.clone();
        let events = control.events.clone();
        let speaking = control.tts_speaking.clone();
        let model_path = control.model_path.clone();
        std::thread::spawn(move || voice::listen_loop(running, events, speaking, model_path));
    }
    Ok(status)
}

#[tauri::command]
async fn get_settings(state: State<'_, Mutex<AppState>>) -> Result<SettingsEntry, String> {
    let guard = state.lock().await;
    Ok(guard.store.settings.clone())
}

#[tauri::command]
async fn save_settings(
    data: serde_json::Map<String, serde_json::Value>,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    let mut merged = match serde_json::to_value(&guard.store.settings) {
        Ok(serde_json::Value::Object(m)) => m,
        _ => return Err("settings serialize".into()),
    };

    for (key, value) in data {
        merged.insert(key, value);
    }

    guard.store.settings = serde_json::from_value(serde_json::Value::Object(merged))
        .map_err(|e| e.to_string())?;

    guard.store.save_settings().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_history(state: State<'_, Mutex<AppState>>) -> Result<Vec<HistoryEntry>, String> {
    let guard = state.lock().await;
    Ok(guard.store.history.clone())
}

#[tauri::command]
async fn get_commands(state: State<'_, Mutex<AppState>>) -> Result<Vec<CommandEntry>, String> {
    let guard = state.lock().await;
    Ok(guard.store.commands.clone())
}

#[tauri::command]
async fn save_command(
    name: String,
    path: String,
    triggers: Vec<String>,
    state: State<'_, Mutex<AppState>>
) -> Result<(), String> {
    let mut guard = state.lock().await;
    guard.store.commands.push(CommandEntry { name, path, triggers });
    guard.store.save_commands().await.map_err(|e| e.to_string())?;
    Ok(())

}

#[tauri::command]
async fn delete_command(
    index: usize,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    if index >= guard.store.commands.len() {
        return Err("index out of range".into())
    }
    guard.store.commands.remove(index);
    guard.store.save_commands().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn toggle_maximize(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.is_maximized().map_err(|e| e.to_string())? {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn minimize_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
fn move_window(x: i32, y: i32, window: tauri::WebviewWindow) -> Result<(), String> {
    window.set_position(tauri::PhysicalPosition {x, y}).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

fn main() {
    // whisper log-flooding fix
    whisper_rs::install_logging_hooks();
    // webkit + wayland + nvidia fix
    unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1"); }

    // tx goes to dispatcher, rx to read it
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (voice_tx, mut voice_rx) = tokio::sync::mpsc::unbounded_channel::<voice::VoiceEvent>();

    let store = tauri::async_runtime::block_on(MemoryStore::load())
        .expect("не вдалось завантажити memory/");
    let dispatcher = Dispatcher::new(tx);
    let voice = store.settings.map.get("tts_voice")
        .and_then(|v| v.as_str()).unwrap_or("").to_string();
    let rate = store.settings.map.get("tts_rate")
        .and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let tts_speaking = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let tts = tts::spawn_tts(voice, rate, tts_speaking.clone());
    let _ = tts.send(tts::TtsCommand::SetMute(store.settings.mute_status));
    let model_path = store.settings.map.get("model_path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("models/ggml-base.bin").to_string();
    let state = Mutex::new(AppState {dispatcher, store, tts});
    let voice_control = VoiceControl {
       running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
       events: voice_tx,
       tts_speaking,
       model_path,
    };

    tauri::Builder::default()
        .manage(state)
        .manage(voice_control)
        .setup(move |app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut rx = rx;
                while let Some(msg) = rx.recv().await {
                    let _ = handle.emit("reminder", msg);
                }
            });
            let handle2 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Some(ev) = voice_rx.recv().await {
                    match ev {
                        voice::VoiceEvent::Status(s) => {
                            let _ = handle2.emit("setStatus", s);
                        }
                        voice::VoiceEvent::Stop => {
                            let _ = handle2.emit("setStatus", "idle");
                            let state = handle2.state::<Mutex<AppState>>();
                            let guard = state.lock().await;
                            let _ = guard.tts.send(tts::TtsCommand::Stop);
                        }
                        voice::VoiceEvent::Command(cmd) => {
                            let _ = handle2.emit("addMessage", (cmd.clone(), "user"));
                            let state = handle2.state::<Mutex<AppState>>();
                            let mut guard = state.lock().await;
                            let AppState { dispatcher, store, tts } = &mut *guard;
                            let response = dispatcher.execute(&cmd, store).await;
                            if !response.is_empty() {
                                let _ = tts.send(tts::TtsCommand::Speak(response.clone()));
                                let _ = handle2.emit("addMessage", (response, "tom"));
                            }
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            execute_command, toggle_ai_mode, toggle_mute,
            get_settings, save_settings, toggle_maximize,
            get_history, get_commands,
            save_command, delete_command,
            minimize_window, close_window,
            move_window, open_url, toggle_listening,
        ])
        .run(tauri::generate_context!())
        .expect("помилка запуску Tauri")
}