use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use dirs::config_dir;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use nullnet_liberror::{location, Error, ErrorHandler, Location};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Secret {
    ORG_ID,
    APP_ID,
    APP_SECRET,
}

impl Secret {
    fn as_str(&self) -> &'static str {
        match self {
            Secret::ORG_ID => "ORG_ID",
            Secret::APP_ID => "APP_ID",
            Secret::APP_SECRET => "APP_SECRET",
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct ConfigStore {
    values: HashMap<String, String>,
}

pub struct Storage;

static STORAGE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("wallguard");
    path
});

static STORE: Lazy<Mutex<ConfigStore>> = Lazy::new(|| {
    let full_path = Storage::file_path();
    let data = read_to_string(&full_path).ok();
    let values = data
        .and_then(|s| serde_json::from_str::<ConfigStore>(&s).ok())
        .unwrap_or_default();
    Mutex::new(values)
});

impl Storage {
    const FILE_NAME: &'static str = "config.json";

    fn file_path() -> PathBuf {
        let mut path = STORAGE_PATH.clone();
        path.push(Self::FILE_NAME);
        path
    }

    pub fn init() -> Result<(), Error> {
        let dir = STORAGE_PATH.clone();
        let file_path = Self::file_path();

        create_dir_all(&dir).handle_err(location!())?;

        if !file_path.exists() {
            let default = ConfigStore::default();
            let json = serde_json::to_string_pretty(&default).handle_err(location!())?;
            let mut file = File::create(&file_path).handle_err(location!())?;
            file.write_all(json.as_bytes()).handle_err(location!())?;
        }

        Ok(())
    }

    pub fn get_value(secret: Secret) -> Option<String> {
        let store = STORE.lock().ok()?;
        store.values.get(secret.as_str()).cloned()
    }

    pub fn set_value(secret: Secret, value: &str) -> Result<(), Error> {
        let mut store = STORE.lock().handle_err(location!())?;
        store.values.insert(secret.as_str().into(), value.into());
        let json = serde_json::to_string_pretty(&*store).handle_err(location!())?;
        write(Self::file_path(), json).handle_err(location!())
    }
}
