use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSettings {
    pub theme_name: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme_name: "Kanagawa".to_string(),
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    let app_config_dir = config_dir.join("bossy-rust");
    fs::create_dir_all(&app_config_dir)?;
    Ok(app_config_dir.join("settings.toml"))
}

pub fn save_settings(settings: &UserSettings) -> Result<()> {
    let path = get_config_path()?;
    let toml_string = toml::to_string(settings)?;
    fs::write(path, toml_string)?;
    Ok(())
}

pub fn load_settings() -> Result<UserSettings> {
    let path = get_config_path()?;
    if !path.exists() {
        return Ok(UserSettings::default());
    }
    let toml_string = fs::read_to_string(path)?;
    let settings: UserSettings = toml::from_str(&toml_string)?;
    Ok(settings)
}
