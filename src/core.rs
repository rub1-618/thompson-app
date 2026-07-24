use std::{io::{Error, ErrorKind::{InvalidData, NotFound}}};
use tokio::fs::{create_dir_all, read_to_string, rename, write};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{to_string_pretty};

const COMMANDS_PATH: &str = "./memory/commands.json";
const HISTORY_PATH: &str = "./memory/history.json";
const SETTINGS_PATH: &str = "./memory/settings.json";

const HISTORY_CAP: usize = 100;

pub struct MemoryStore {
    pub settings: SettingsEntry,
    pub history: Vec<HistoryEntry>,
    pub commands: Vec<CommandEntry>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsEntry {
    pub accent: String,
    pub bg: String,
    pub muted: String,
    
    pub local_model: String,

    #[serde(default)]
    pub ai_mode: String,

    #[serde(default)]
    pub gemini_key: String,

    #[serde(default)]
    pub mute_status: bool,
    
    #[serde(flatten)]
    pub map: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub role: Roles,
    pub text: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub name: String,
    pub path: String,
    pub triggers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Roles {
    User,
    Tom,
}

pub async fn load_json<T: DeserializeOwned + Default>(path: &str) -> Result<T, Error> {
    let text = match read_to_string(path).await {
        Ok(s) => s,
        Err(e) => {
            if e.kind() == NotFound {
                return Ok(T::default())
            } else {  
                return Err(e)
            }
        }
    };
    serde_json::from_str(&text).map_err(|e| Error::new(InvalidData, e))
}

pub async fn save_json<T: Serialize>(path: &str, value: &T) -> Result<(), Error> {
    let text = to_string_pretty(value).map_err(|e| Error::new(InvalidData, e))?;
    let tmp_path = format!("{}.tmp", path);
    write(&tmp_path, text).await?;
    rename(&tmp_path, path).await
}

impl MemoryStore {
    pub async fn load() ->  Result<Self, Error> {
        create_dir_all("./memory").await?;
        Ok(MemoryStore { 
            settings: load_json(SETTINGS_PATH).await?, 
            history: load_json(HISTORY_PATH).await?, 
            commands: load_json(COMMANDS_PATH).await?,
        })
    }

    pub async fn save_settings(&self) -> Result<(), Error> {
        save_json(SETTINGS_PATH, &self.settings).await
    }

    pub async fn save_commands(&self) -> Result<(), Error> {
       save_json(COMMANDS_PATH, &self.commands).await
    }

    pub async fn save_history(&mut self) -> Result<(), Error> {
        if self.history.len() > HISTORY_CAP {
            self.history.drain(..self.history.len() - HISTORY_CAP);
        }
        save_json(HISTORY_PATH, &self.history).await
    }
}