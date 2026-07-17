use crate::core::{HistoryEntry, MemoryStore, Roles};
use std::io::Error;

mod ai;
mod auto;
mod core;
mod tts;
mod voice;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut store = MemoryStore::load().await?;

    let question = "Привіт! Як тебе звати?";
    let answer = ai::ask_ai(question, &store.history, &store.settings).await;
    println!("Q: {question}");
    println!("A: {answer}");

    store.history.push(HistoryEntry { role: Roles::User, text: question.to_string() });
    store.history.push(HistoryEntry { role: Roles::Tom, text: answer });
    store.save_history().await?;

    Ok(())
}
