use crate::core::MemoryStore;
use crate::dispatcher::Dispatcher;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex;

mod ai;
mod auto;
mod commands;
mod core;
mod dispatcher;
mod tts;
mod voice;

/// Единое разделяемое состояние: диспетчер (со своим pending-диалогом)
/// и хранилище памяти. Под Mutex, потому что команды моста могут прийти
/// конкурентно, а execute требует &mut both.
struct AppState {
    dispatcher: Dispatcher,
    store: MemoryStore,
}

/// Мост: фронт зовёт invoke("execute_command", { text }), сюда приходит
/// text, гоним его через тот же dispatcher.execute, что и REPL раньше.
#[tauri::command]
async fn execute_command(
    text: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let mut guard = state.lock().await;
    let AppState { dispatcher, store } = &mut *guard;
    Ok(dispatcher.execute(&text, store).await)
}

fn main() {
    // Канал напоминаний: tx уезжает в диспетчер (пишут таймеры),
    // rx — читатель, который теперь шлёт событие во фронт вместо println.
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Загружаем состояние до старта UI. block_on — на верхушке main
    // никакой рантайм ещё не крутится, вложенности нет.
    let store = tauri::async_runtime::block_on(MemoryStore::load())
        .expect("не вдалось завантажити memory/");
    let dispatcher = Dispatcher::new(tx);
    let state = Mutex::new(AppState { dispatcher, store });

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            // rx переезжает в фоновую задачу; ей нужен AppHandle, чтобы
            // достучаться до фронта (у неё нет общего стека с командами).
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut rx = rx;
                while let Some(msg) = rx.recv().await {
                    let _ = handle.emit("reminder", msg);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![execute_command])
        .run(tauri::generate_context!())
        .expect("помилка запуску Tauri");
}
