use serde::{Deserialize, Serialize};
use crate::money::Money;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct RouletteStatistics {
    pub spins_won: u32,
    pub spins_lost: u32,
    pub money_won: Money,
    pub money_lost: Money,
    pub biggest_win: Money,
    pub biggest_loss: Money,
}

impl RouletteStatistics {
    pub fn record_win(&mut self, amount: Money) {
        self.spins_won += 1;
        self.money_won += amount;
        if amount > self.biggest_win {
            self.biggest_win = amount;
        }
    }

    pub fn record_loss(&mut self, amount: Money) {
        self.spins_lost += 1;
        self.money_lost += amount;
        if amount > self.biggest_loss {
            self.biggest_loss = amount;
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct BlackjackStatistics {
  pub hands_won: u32,
  pub hands_lost: u32,
  pub hands_push: u32,
  pub money_won: Money,
  pub money_lost: Money,
  pub biggest_win: Money,
  pub biggest_loss: Money,
  pub biggest_bankroll: Money,
  pub times_bankrupted: u32,
}

impl BlackjackStatistics {
  pub fn record_win(&mut self, amount: Money) {
    self.hands_won += 1;
    self.money_won += amount;
    if amount > self.biggest_win {
      self.biggest_win = amount;
    }
  }

  pub fn record_loss(&mut self, amount: Money) {
    self.hands_lost += 1;
    self.money_lost += amount;
    if amount > self.biggest_loss {
      self.biggest_loss = amount;
    }
  }

  pub fn record_push(&mut self) {
    self.hands_push += 1;
  }

  pub fn update_bankroll(&mut self, amount: Money) {
    if amount > self.biggest_bankroll {
      self.biggest_bankroll = amount;
    } else if amount.is_zero() {
      self.times_bankrupted += 1;
    }
  }
}
