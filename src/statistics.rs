use std::{
    fs::{self, write},
    path::Path,
};

use crate::money::Money;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Statistics {
    #[serde(default)]
    pub biggest_bankroll: Money,
    #[serde(default)]
    pub times_bankrupted: u32,
    #[serde(default)]
    pub blackjack: BlackjackStatistics,
    #[serde(default)]
    pub roulette: RouletteStatistics,
    #[serde(default)]
    pub slots: SlotMachineStatistics,
}

impl Statistics {
    pub fn init(stats_path: &Path) -> Result<()> {
        if !stats_path.try_exists()? {
            let stats = Self::default();
            stats.save(stats_path)?;
        }
        Ok(())
    }

    pub fn load(stats_path: &Path) -> Result<Self> {
        let stats_string = fs::read_to_string(stats_path)?;

        let stats = toml::from_str(&stats_string)?;

        Ok(stats)
    }

    pub fn save(&self, stats_path: &Path) -> Result<()> {
        fs::create_dir_all(stats_path.parent().unwrap()).expect("Couldn't create stats directory!");
        write(stats_path, toml::to_string(&self)?)?;
        Ok(())
    }

    pub fn update_bankroll(&mut self, amount: Money) {
        if amount > self.biggest_bankroll {
            self.biggest_bankroll = amount;
        } else if amount.is_zero() {
            self.times_bankrupted += 1;
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SlotMachineStatistics {
    pub total_pulls: u32,
    pub money_won: Money,
    pub money_spent: Money,
    pub biggest_jackpot: Money,
}

impl SlotMachineStatistics {
    pub fn record_pull(&mut self, amount: Money) {
        self.total_pulls += 1;
        self.money_spent += amount;
    }

    pub fn record_win(&mut self, amount: Money) {
        self.money_won += amount;
        if self.biggest_jackpot < amount {
            self.biggest_jackpot = amount;
        }
    }
}

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
}
