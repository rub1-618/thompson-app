use crate::core::{MemoryStore, load_json, save_json, SettingsEntry, CommandEntry, HistoryEntry, Roles};
use std::io::Error;

mod ai;
mod auto;
mod core;
mod tts;
mod voice;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut store = MemoryStore::load().await?;
    store.history.push(HistoryEntry { role: Roles::User, text: "hello".to_string() });
    store.save_history().await?;
    store.settings.local_model = "llama3".to_string();
    store.save_settings().await?;

    Ok(())
}
