use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use directories::{ProjectDirs};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::blackjack::BlackjackConfig;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
  #[serde(default)]
  pub blackjack: BlackjackConfig,
  #[serde(with = "rust_decimal::serde::str")]
  #[serde(default = "Config::default_greens_gift")]
  pub mister_greens_gift: Decimal,
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
    Ok(fs::write(&config_path, toml::to_string(&self)?)?)
  }

  pub fn init_get() -> Result<Self> {
    let path = Self::default_path();

    if let Ok(config) = Self::from_path(&path) {
      return Ok(config)
    } else {
      let config = Self::default();
      config.save(&path)?;

      return Ok(config)
    }
  }

  fn default_greens_gift() -> Decimal {
    Decimal::new(1_000, 0)
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
