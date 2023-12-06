use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlackjackConfig {
  #[serde(default = "BlackjackConfig::default_shoe_count")]
  pub shoe_count: u8,

  #[serde(default = "BlackjackConfig::default_shuffle_penetration")]
  pub shuffle_at_penetration: f32,
}

impl Default for BlackjackConfig {
  fn default() -> Self {
    Self {
      shoe_count: Self::default_shoe_count(),
      shuffle_at_penetration: Self::default_shuffle_penetration(),
    }
  }
}

impl BlackjackConfig {
  fn default_shoe_count() -> u8 {
    4
  }

  fn default_shuffle_penetration() -> f32 {
    0.75
  }
}
