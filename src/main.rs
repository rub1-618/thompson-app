use crate::{core::{SettingsEntry, HistoryEntry, CommandEntry}, dispatcher::Dispatcher};
use core::MemoryStore;
use tauri::{Emitter, State};
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
}

#[tauri::command]
async fn execute_command(
    text: String, 
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let mut guard = state.lock().await;
    let AppState { dispatcher, store } = &mut *guard;
    Ok(dispatcher.execute(&text, store).await)
}

#[tauri::command]
async fn toggle_mute(
    state: State<'_, Mutex<AppState>>,
) -> Result<bool, String> {
    let mut guard = state.lock().await;
    guard.store.settings.mute_status = !guard.store.settings.mute_status;
    guard.store.save_settings().await.map_err(|e| e.to_string())?;
    Ok(guard.store.settings.mute_status)
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
    // webkit + wayland + nvidia fix
    unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1"); }

    // tx goes to dispatcher, rx to read it
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let store = tauri::async_runtime::block_on(MemoryStore::load())
        .expect("не вдалось завантажити memory/");
    let dispatcher = Dispatcher::new(tx);
    let state = Mutex::new(AppState {dispatcher, store});

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut rx = rx;
                while let Some(msg) = rx.recv().await {
                    let _ = handle.emit("reminder", msg);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            execute_command, toggle_mute,
            get_settings, save_settings, 
            get_history, get_commands,
            save_command, delete_command,
            minimize_window, close_window,
            move_window, open_url,
        ])
        .run(tauri::generate_context!())
        .expect("помилка запуску Tauri")
}