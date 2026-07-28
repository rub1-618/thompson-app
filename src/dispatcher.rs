use crate::ai;
use crate::core::{HistoryEntry, MemoryStore, Roles};
use crate::commands::{CMDS, Command};
use rapidfuzz::fuzz::ratio;
use tokio::sync::mpsc::UnboundedSender;
use std::collections::BTreeSet;
use std::u64;

const CMD_THRESHOLD: f64 = 0.55;
const CANCEL_PHRASES: [&str; 6] = ["ні", "no", "скасуй", "відміни", "не треба", "cancel" ];
const CONFIRM_PHRASES: [&str; 8] = ["так", "yes", "підтверджую", "підтверди", "ок", "ok", "добре", "гаразд"];
const CANCEL_THRESHOLD: f64 = 0.8;
const CONFIRM_THRESHOLD: f64 = 0.8;

pub struct  Dispatcher {
    pending: PendingDialog,
    events: tokio::sync::mpsc::UnboundedSender<String>,
}

pub enum PendingDialog {
    None, 
    Dictation,
}

impl Dispatcher {
    pub fn new(tx: UnboundedSender<String>) -> Self {
        Dispatcher { pending: PendingDialog::None, events: tx }
    }

    pub async fn execute(&mut self, text: &str, store: &mut MemoryStore) -> String {
        let pending = std::mem::replace(&mut self.pending, PendingDialog::None);
        let response = match pending {
            PendingDialog::None => self.dispatch(text, store).await,
            PendingDialog::Dictation => continue_dictation(text),
        };

        if !response.is_empty() {
            store.history.push(HistoryEntry { role: Roles::User, text: text.to_string() });
            store.history.push(HistoryEntry { role: Roles::Tom, text: response.clone() });
            if let Err(e) = store.save_history().await {
                eprintln!("history save failed: {e}");
            }
        }

        response
    }

    async fn dispatch(&mut self, text: &str, store: &mut MemoryStore) -> String {
        if match_whole_phrase(text, &CONFIRM_PHRASES, CONFIRM_THRESHOLD) {
             return "Підтверджено.".to_string(); 
        }

        if match_whole_phrase(text, &CANCEL_PHRASES, CANCEL_THRESHOLD) {
             return "Скасовано.".to_string(); 
        }

        if let Some((cmd, _)) = find_command(text) {
            return match cmd {
                Command::Screen => crate::auto::screen(&store.settings).await,
                _ => return self.run_builtin(cmd, text)
            }
        }

        if let Some((name, path)) = find_custom(text, store) {
            return run_custom(&name, &path);
        }

        ai::ask_ai(text, &store.history, &store.settings).await
    }

    fn run_builtin(&mut self, cmd: Command, text: &str) -> String {
        match cmd {
            Command::Stop => String::new(),
            
            Command::Ctime => chrono::Local::now()
            .format("Зараз %H:%M, %d.%m.%Y.").to_string(),

            Command::Remind => {
                let (minutes, label) = parse_reminder(text);
                let tx = self.events.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(minutes * 60)).await;
                    let _ = tx.send(label);
                });
                format!("Нагадаю через {} хв.", minutes)
            }

            Command::OpenBrowser => {
                let _ = open::that("https://google.com");
                "Браузер відкрито!".to_string()
            }

            Command::Stats => crate::auto::stats(),

            Command::Dictation => {
                let inline = strip_dictation_trigger(text);
                if !inline.is_empty() {
                    continue_dictation(inline)
                } else {
                    self.pending = PendingDialog::Dictation;
                    "Що надрукувати?".to_string()
                }
            }
            
            Command::WindowOpen | Command::WindowHide
            | Command::WindowClose => crate::auto::window(text),
            Command::Screen => String::new(),
            Command::MusicToggle => crate::auto::music_toggle(),
            Command::MusicNext => crate::auto::music_next(),
            Command::MusicPrev => crate::auto::music_prev(),
            Command::MusicInfo => crate::auto::music_info(),
            Command::ToggleSound => crate::auto::sound_toggle(),
        }
    }
}

fn find_custom(text: &str, store: &MemoryStore) -> Option<(String, String)> {
    for cmd in &store.commands {
        for trigger in &cmd.triggers {
            if  token_set_ratio(text, trigger) >= CMD_THRESHOLD {
                return Some((cmd.name.clone(), cmd.path.clone()))
            }
        }
    }
    None
}

fn run_custom(name: &str, path: &str) -> String {
    match tokio::process::Command::new("sh").arg("-c").arg(path).spawn() {
        Ok(_) => format!("Виконую '{name}'."),
        Err(_) => format!("Не вдалось виконати '{name}'."),
    }
}

fn parse_reminder(text: &str) -> (u64, String) {
    const SKIP: [&str; 18] = ["нагади", "нагадай", "нагадування", 
        "remind",  "me", "reminder", "встанови", "нагадування", 
        "через", "in", "after", "хв", "хвилин", "min", "mins",
        "minutes", "хвилину", "хвилини"];
    let mut minutes: Option<u64> = None;
    let mut label_words: Vec<&str> = Vec::new();
    for word in text.split_whitespace() {
        if minutes.is_none() {
            if let Ok(n) = word.parse::<u64>() {
                minutes = Some(n);
                continue;
            }
        }
        if !SKIP.contains(&word.to_lowercase().as_str()) {
            label_words.push(word);
        }
    }

    let minutes = minutes.unwrap_or(5);
    let label = if label_words.is_empty() {
        "Час минув!".to_string()
    } else {
        label_words.join(" ")
    };
    (minutes, label)
}

fn strip_dictation_trigger(text: &str) -> &str {
    const TRIGGERS: [&str; 6] = ["напиши", "надрукуй", "введи текст", "type this", "print this", "type"];
    let lower= text.to_lowercase();
    for trigger in TRIGGERS {
        if lower.starts_with(trigger) {
            return text[trigger.len()..].trim_start()
        }
    }
    ""
}

fn continue_dictation(text: &str) -> String {
    if match_whole_phrase(text, &CANCEL_PHRASES, CANCEL_THRESHOLD) {
            return "Скасовано.".to_string();
    }
    format!("Надруковано: {text}")
}

pub fn find_command(text: &str) -> Option<(Command, f64)> {
    let mut best: Option<(Command, f64)> = None;
    let text = &text.to_lowercase();
    for (command, phrases) in CMDS {
        for phrase in *phrases {
            let score =  0.7 * token_set_ratio(text, *phrase) +
                0.3 * ratio(text.chars(), phrase.chars());

            let current_best = best.map_or(0.0, |(_, s)| s);
            if score > current_best {
                best = Some((*command, score));
            }
        }
    }

    best.filter(|(_, score)| *score >= CMD_THRESHOLD)
}

pub fn match_whole_phrase(text: &str, phrases: &[&str], threshold: f64) -> bool {
    let text = text.trim().to_lowercase();
    for phrase in phrases {
        if ratio(text.chars(), phrase.chars()) >= threshold {
            return true
        }
    }
    false
}

pub fn token_set_ratio(str1: &str, str2: &str) -> f64 {
    let s1 = str1.to_lowercase();
    let s2 = str2.to_lowercase();
    let set1: BTreeSet<&str> = s1.split_whitespace().collect();
    let set2: BTreeSet<&str> = s2.split_whitespace().collect();

    let common: BTreeSet<&str> = set1.intersection(&set2).copied().collect();
    let diff1: BTreeSet<&str> = set1.difference(&set2).copied().collect();
    let diff2: BTreeSet<&str> = set2.difference(&set1).copied().collect();

    let t0: Vec<&str> = common.iter().copied().collect();
    let t1: Vec<&str> = common.iter().copied().chain(diff1.iter().copied()).collect();
    let t2: Vec<&str> = common.iter().copied().chain(diff2.iter().copied()).collect();

    let strt0 = t0.join(" ");
    let strt1 = t1.join(" ");
    let strt2 = t2.join(" ");

    let ratio0 = ratio(strt0.chars(), strt1.chars());
    let ratio1 = ratio(strt0.chars(), strt2.chars());
    let ratio2 = ratio(strt1.chars(), strt2.chars());

    f64::max(ratio0, f64::max(ratio1, ratio2))
}

#[cfg(test)]
mod tests {
use rapidfuzz::{fuzz::ratio};
use crate::{commands::Command, dispatcher::{CMD_THRESHOLD, find_command, parse_reminder, strip_dictation_trigger, token_set_ratio}};

    #[test]
    fn test_tsr() {
        let str1 = "плагін запусти";
        let str2 = "запусти плагін";

        assert!(token_set_ratio(str1, str2) == 1.0)
    }

    #[test]
    fn test_tsr_neg() {
        let str1 = "котра година";
        let str2 = "запусти плагін";

        assert!(token_set_ratio(str1, str2) < 0.3)
    }

    #[test]
    fn test_formula() {
        let str1 = "томікс ну запусти плагін будь ласка";
        let str2 = "запусти плагін";

        assert!(0.7 * token_set_ratio(str1, str2) + 0.3 * ratio(str1.chars(), str2.chars()) >= CMD_THRESHOLD)
    }

    #[test]
    fn test_findcmd() {
        match find_command("стоп") {
            Some((cmd, score)) => {
                assert_eq!(cmd, Command::Stop);
                assert!((score - 1.0).abs() < 1e-9)
            }
            None => panic!("Expected command."),
        }
    }

    #[test]
    fn test_cmds() {
        let cases = [
            ( "стоп",               Command::Stop ),
            ( "котра година",       Command::Ctime ),
            ( "статистика",         Command::Stats ),
            ( "що на екрані",       Command::Screen ),
            ( "музика",             Command::MusicToggle ),
            ( "наступний трек",     Command::MusicNext ),
            ( "попередній трек",    Command::MusicPrev ),
            ( "який трек",          Command::MusicInfo ),
            ( "нагадування",        Command::Remind ),
            ( "відкрий браузер",    Command::OpenBrowser ),
            ( "увімкнути звук",     Command::ToggleSound ),
            // todo dictation, windows
        ];
        for ( phrase, expected ) in cases {
            let (cmd, _) = find_command(phrase).unwrap_or_else(|| panic!("No match for a command!"));
            assert_eq!(cmd, expected);
        }
    }

    #[test]
    fn test_dt_inline() {
        assert_eq!(strip_dictation_trigger("type hello world"), "hello world")
    }

    #[test]
    fn test_dt_none() {
        assert_eq!(strip_dictation_trigger("print this"), "")
    }

    #[test]
    fn test_dt_highcase() {
        assert_eq!(strip_dictation_trigger("Type HELLO WORLD"), "HELLO WORLD")
    }
    
    #[test]
    fn test_dt_no_trigger() {
        assert_eq!(strip_dictation_trigger("hello"), "")
    }

    #[test]
    fn test_reminder() {
        assert_eq!(
            parse_reminder("remind me in 5 minutes to drink tea"),
            (5, "to drink tea".to_string())
        );
    }

    #[test]
    fn test_reminder_no_label() {
        assert_eq!(
            parse_reminder("remind me in 5 mins"),
            (5, "Час минув!".to_string())
        );
    }

    #[test]
    fn test_reminder_no_time() {
        assert_eq!(
            parse_reminder("remind me to drink tea"),
            (5, "to drink tea".to_string())
        );
    }
}