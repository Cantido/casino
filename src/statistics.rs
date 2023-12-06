use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct BlackjackStatistics {
  pub hands_won: u32,
  pub hands_lost: u32,
  pub hands_push: u32,
  #[serde(with = "rust_decimal::serde::str")]
  pub money_won: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  pub money_lost: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  pub biggest_win: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  pub biggest_loss: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  pub biggest_bankroll: Decimal,
  pub times_bankrupted: u32,
}

impl BlackjackStatistics {
  pub fn record_win(&mut self, amount: Decimal) {
    self.hands_won += 1;
    self.money_won += amount;
    if amount > self.biggest_win {
      self.biggest_win = amount;
    }
  }

  pub fn record_loss(&mut self, amount: Decimal) {
    self.hands_lost += 1;
    self.money_lost += amount;
    if amount > self.biggest_loss {
      self.biggest_loss = amount;
    }
  }

  pub fn record_push(&mut self) {
    self.hands_push += 1;
  }

  pub fn update_bankroll(&mut self, amount: Decimal) {
    if amount > self.biggest_bankroll {
      self.biggest_bankroll = amount;
    } else if amount.is_zero() {
      self.times_bankrupted += 1;
    }
  }
}
