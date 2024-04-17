use crate::blackjack::BlackjackConfig;
use crate::money::Money;
use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub blackjack: BlackjackConfig,
    #[serde(default = "Config::default_greens_gift")]
    pub mister_greens_gift: Money,
    #[serde(default = "Config::default_save_path")]
    pub save_path: PathBuf,
    #[serde(default = "Config::default_stats_path")]
    pub stats_path: PathBuf,
}

impl Config {
    pub fn default_path() -> PathBuf {
        let project_dirs = Self::project_dirs();
        let config_dir = project_dirs.config_dir();
        config_dir.join("config.toml")
    }

    pub fn from_path(config_path: &Path) -> Result<Self> {
        let config_string = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&config_string)?)
    }

    pub fn save(&self, config_path: &Path) -> Result<()> {
        fs::create_dir_all(config_path.parent().ok_or(anyhow!(
            "Configuraton file path doesn't have a parent we can create!"
        ))?)
        .expect("Couldn't create save dir!");
        Ok(fs::write(&config_path, toml::to_string(&self)?)?)
    }

    pub fn init_get() -> Result<Self> {
        let path = Self::default_path();

        if path.try_exists()? {
            Self::from_path(&path)
                .with_context(|| "Failed to load config from file")
        } else {
            let dir = path.parent().expect("Expected config path to have a parent directory.");
            fs::create_dir_all(dir)?;

            let config = Self::default();
            config.save(&path)
                .with_context(|| "Failed to initialize config file")?;
            Ok(config)
        }
    }

    fn default_greens_gift() -> Money {
        Money::from_major(1_000)
    }

    fn default_save_path() -> PathBuf {
        let project_dirs = Self::project_dirs();
        let data_dir = project_dirs.data_dir();
        data_dir.join("state.toml")
    }

    fn default_stats_path() -> PathBuf {
        let project_dirs = Self::project_dirs();
        let data_dir = project_dirs.data_dir();
        data_dir.join("stats.toml")
    }

    fn project_dirs() -> ProjectDirs {
        ProjectDirs::from("dev", "Cosmicrose", "casino").unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blackjack: Default::default(),
            mister_greens_gift: Self::default_greens_gift(),
            save_path: Self::default_save_path(),
            stats_path: Self::default_stats_path(),
        }
    }
}
