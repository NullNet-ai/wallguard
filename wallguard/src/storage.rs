use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::{File, create_dir_all, read_to_string, write};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

// @TODO
// use dirs::config_dir;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use nullnet_liberror::{Error, ErrorHandler, Location, location};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Secret {
    InstallationCode,
    AppId,
    AppSecret,
}

impl Secret {
    fn as_str(&self) -> &'static str {
        match self {
            Secret::InstallationCode => "InstallationCode",
            Secret::AppId => "AppId",
            Secret::AppSecret => "AppSecret",
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct ConfigStore {
    values: HashMap<String, String>,
}

pub struct Storage;

static STORAGE_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("/root/.config").join("wallguard"));

static STORE: Lazy<Mutex<Option<ConfigStore>>> = Lazy::new(|| Mutex::new(None));

impl Storage {
    const FILE_NAME: &'static str = "config.json";

    fn file_path() -> PathBuf {
        let mut path = STORAGE_PATH.clone();
        path.push(Self::FILE_NAME);
        path
    }

    pub async fn init() -> Result<(), Error> {
        let dir = STORAGE_PATH.clone();
        let file_path = Self::file_path();

        create_dir_all(&dir).await.handle_err(location!())?;

        let config = if file_path.exists() {
            read_to_string(&file_path)
                .await
                .ok()
                .and_then(|s| serde_json::from_str::<ConfigStore>(&s).ok())
                .unwrap_or_default()
        } else {
            let default = ConfigStore::default();
            let json = serde_json::to_string_pretty(&default).handle_err(location!())?;
            let mut file = File::create(&file_path).await.handle_err(location!())?;
            file.write_all(json.as_bytes())
                .await
                .handle_err(location!())?;
            default
        };

        let mut store = STORE.lock().await;
        *store = Some(config);
        Ok(())
    }

    pub async fn get_value(secret: Secret) -> Option<String> {
        let store = STORE.lock().await;
        store.as_ref()?.values.get(secret.as_str()).cloned()
    }

    pub async fn set_value(secret: Secret, value: &str) -> Result<(), Error> {
        let mut store = STORE.lock().await;
        let config = store
            .as_mut()
            .ok_or("Storage not initialized")
            .handle_err(location!())?;

        config.values.insert(secret.as_str().into(), value.into());
        let json = serde_json::to_string_pretty(&*config).handle_err(location!())?;
        write(Self::file_path(), json).await.handle_err(location!())
    }

    pub async fn delete_value(secret: Secret) -> Result<(), Error> {
        let mut store = STORE.lock().await;
        let config = store
            .as_mut()
            .ok_or("Storage not initialized")
            .handle_err(location!())?;

        config.values.remove(secret.as_str());
        let json = serde_json::to_string_pretty(&*config).handle_err(location!())?;
        write(Self::file_path(), json).await.handle_err(location!())
    }
}
