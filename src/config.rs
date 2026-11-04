use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::i18n::Lang;
use crate::secure_store;

fn default_language() -> String {
    "en".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_id: String,
    pub api_hash: String,
    pub phone: String,
    #[serde(default)]
    pub output_dir: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub disclaimer_version: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_id: String::new(),
            api_hash: String::new(),
            phone: String::new(),
            output_dir: String::new(),
            language: default_language(),
            disclaimer_version: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConfigLoadResult {
    pub config: AppConfig,
    pub vault_error: Option<String>,
}

impl AppConfig {
    pub fn vault_path(base: &PathBuf) -> PathBuf {
        base.join(".tguser_config.enc")
    }

    pub fn legacy_json_path(base: &PathBuf) -> PathBuf {
        base.join(".tguser_config.json")
    }

    /// Load settings from the encrypted vault (migrates legacy JSON once).
    pub fn load(base: &PathBuf) -> Self {
        Self::load_with_status(base).config
    }

    pub fn load_with_status(base: &PathBuf) -> ConfigLoadResult {
        let vault = Self::vault_path(base);
        let legacy = Self::legacy_json_path(base);
        let _ = secure_store::migrate_legacy_file(&legacy, &vault);

        if !vault.is_file() {
            return ConfigLoadResult {
                config: Self {
                    output_dir: base.join("output").to_string_lossy().into_owned(),
                    ..Default::default()
                },
                vault_error: None,
            };
        }

        match secure_store::load_plain_or_decrypt(&vault) {
            Ok(bytes) => match serde_json::from_slice::<AppConfig>(&bytes) {
                Ok(mut c) => {
                    if c.output_dir.is_empty() {
                        c.output_dir = base.join("output").to_string_lossy().into_owned();
                    }
                    c.language = Lang::from_config(&c.language).to_config().to_string();
                    ConfigLoadResult {
                        config: c,
                        vault_error: None,
                    }
                }
                Err(e) => ConfigLoadResult {
                    config: Self {
                        output_dir: base.join("output").to_string_lossy().into_owned(),
                        ..Default::default()
                    },
                    vault_error: Some(format!("settings file is corrupted: {e}")),
                },
            },
            Err(e) => ConfigLoadResult {
                config: Self {
                    output_dir: base.join("output").to_string_lossy().into_owned(),
                    ..Default::default()
                },
                vault_error: Some(format!(
                    "could not decrypt settings (keep .tguser_vault.key with .tguser_config.enc): {e}"
                )),
            },
        }
    }

    pub fn save(&self, base: &PathBuf) -> Result<()> {
        let mut to_save = self.clone();
        to_save.language = Lang::from_config(&to_save.language).to_config().to_string();
        let json = serde_json::to_vec(&to_save)?;
        secure_store::save_encrypted(&Self::vault_path(base), &json)
            .context("save encrypted config")
    }

    pub fn lang(&self) -> Lang {
        Lang::from_config(&self.language)
    }

    pub fn parsed_api_id(&self) -> Result<i32> {
        self.api_id
            .trim()
            .parse()
            .context("API ID must be a number")
    }
}
