use core::fmt;
use std::{io::{stdout, Write}, thread::sleep, time::Duration};

use anyhow::Result;
use colored::*;
use crossterm::{cursor, terminal, QueueableCommand};
use inquire::Select;
use itertools::Itertools;
use rand::{distributions::{Distribution, WeightedIndex}, thread_rng, Rng};
use rust_decimal::Decimal;

use crate::{blackjack::Casino, money::Money};

pub fn play_slots() -> Result<()> {
    let mut casino = Casino::from_filesystem()?;

    let options = vec![
        PriceTier::new(Money::from_major(1)),
        PriceTier::new(Money::from_major(10)),
        PriceTier::new(Money::from_major(100)),
        PriceTier::new(Money::from_major(1_000)),
        PriceTier::new(Money::from_major(5_000))
    ];
    let bet_selection = Select::new(format!("Which slot machine to use? (you have {}) ", casino.bankroll).as_str(), options).prompt().unwrap();
    let bet_amount = bet_selection.cost;

    casino.bankroll -= bet_amount;
    casino.save();
    println!("{}", format!("* You insert your money into the {bet_amount} slot machine.").dimmed());
    println!("You now have {} in the bank", casino.bankroll);
    sleep(Duration::from_millis(600));
    println!("{}", "* You pull the arm of the slot machine.".dimmed());
    sleep(Duration::from_millis(600));
    println!("{}", "* The wheels start spinning.".dimmed());

    let slot_machine = SlotMachine::new_with_default_symbols(bet_selection.multiplier);

    let mut rng = thread_rng();
    let mut position = 0.0;
    let mut velocity = rng.gen_range(20.0..40.0);
    let accel = rng.gen_range(-10.0..-5.0);

    let mut stdout = stdout();

    let mut selected: Vec<&Symbol> = slot_machine.pull();

    while velocity > 0.0 {
        if position >= 1.0 {
            selected = slot_machine.pull();
            position -= 1.0;
        }

        stdout.queue(cursor::SavePosition).unwrap();

        stdout.write_all(format!("▶ {}{}{}{}{} ◀", selected[0], selected[1], selected[2], selected[3], selected[4]).as_bytes()).unwrap();

      stdout.queue(cursor::RestorePosition).unwrap();
      stdout.flush().unwrap();

      sleep(Duration::from_millis(16));

      velocity += accel * (16.0 / 1000.0);
      position += velocity * (16.0 / 1000.0);

      stdout.queue(cursor::RestorePosition).unwrap();
      stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
    }

    stdout.write_all(format!("▶ {}{}{}{}{} ◀", selected[0], selected[1], selected[2], selected[3], selected[4]).as_bytes()).unwrap();
    println!();


    let mut total_payout = Money::ZERO;

    let pay_table = slot_machine.payout(selected);

    if !pay_table.is_empty() {
        println!();
    }

    for entry in pay_table.iter() {
        println!("  {} × {} = {}", entry.symbol, entry.count, entry.payout);
        total_payout += entry.payout;
    }

    println!();
    println!("Payout: {}", total_payout);

    casino.bankroll += total_payout;

    casino.save();

    Ok(())
}

type Symbol = char;
type Weight = u32;

#[derive(Clone, Debug)]
pub struct SlotMachine {
    multiplier: f32,
    weights: Vec<(Symbol, Weight)>,
    distribution: WeightedIndex<Weight>,
}

impl SlotMachine {
    pub fn new_with_default_symbols(multiplier: f32) -> Self {
        let symbols = vec![
            ('🍋', 30),
            ('🍒', 30),
            ('🍊', 30),
            ('🍉', 30),
            ('🔔', 20),
            ('🍌', 20),
            ('🍫', 10),
            ('💰', 2),
            ('💎', 1),
        ];
        let weights: Vec<u32> = symbols.iter().map(|s| s.1).collect();
        Self {
            multiplier,
            weights: symbols,
            distribution: WeightedIndex::new(weights).unwrap(),
        }
    }

    pub fn add_symbol(&mut self, symbol: char, weight: Weight) {
        self.weights.push((symbol, weight));
        self.distribution = WeightedIndex::new(self.weights.iter().map(|i| i.1)).unwrap();
    }

    pub fn payout(&self, symbols: Vec<&Symbol>) -> Vec<PayTableEntry> {
        let mut entries = vec![];

        let counts = symbols.iter().counts();

        for (symbol, count) in counts.iter() {
            if *count >= 3 {
                let sym: char = ***symbol;
                let sym_weight = self.weights.iter().find(|(s, _w)| s == &sym).unwrap().1;
                let sym_value = (self.multiplier * 120.0 / sym_weight as f32) as i64;
                let sym_payout = Money::from_major(sym_value * (count - 2) as i64);

                entries.push(PayTableEntry::new(sym, *count, sym_payout));
            }
        }

        entries
    }

    pub fn pull(&self) -> Vec<&Symbol> {
        let mut rng = thread_rng();
        let samples: Vec<usize> = self.distribution.clone().sample_iter(&mut rng).take(5).collect();
        samples.iter().map(|i| &self.weights[*i].0).collect()
    }
}

pub struct SlotMachineOutput {
    pub entries: Vec<PayTableEntry>,
}

impl SlotMachineOutput {
}

pub struct PayTableEntry {
    pub symbol: Symbol,
    pub count: usize,
    pub payout: Money,
}

impl PayTableEntry {
    pub fn new(symbol: Symbol, count: usize, payout: Money) -> Self {
        Self {
            symbol,
            count,
            payout,
        }
    }
}

struct PriceTier {
    pub cost: Money,
    pub multiplier: f32,
}

impl PriceTier {
    pub fn new(cost: Money) -> Self {
        let mult: Decimal = cost.try_into().unwrap();
        Self { cost, multiplier: mult.try_into().unwrap()  }
    }
}

impl fmt::Display for PriceTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} per pull", self.cost)
    }
}

#[cfg(test)]
mod test {
    use rust_decimal::Decimal;

    use crate::{money::Money, slots::{PayTableEntry, SlotMachine}};

    #[test]
    fn test_symbols_return_to_player() {
        let slot_machine = SlotMachine::new_with_default_symbols(1.0);

        let mut total_player_payment = Money::ZERO;
        let mut total_player_return = Money::ZERO;

        for _i in 1..10_000 {
            total_player_payment += Money::from_major(1);

            let payout: Vec<PayTableEntry> = slot_machine.payout(slot_machine.pull());

            for pay_table_entry in payout.iter() {
                total_player_return += pay_table_entry.payout;
            }
        }

        let total_return: Decimal = total_player_return.into();
        let total_payment: Decimal = total_player_payment.into();
        let rtp_ratio: f32 = (total_return / total_payment).try_into().unwrap();

        assert!(rtp_ratio >= 0.90, "Return-to-player ratio is {rtp_ratio}, which should be higher than 0.90");
        assert!(rtp_ratio < 1.0, "Return-to-player ratio is {rtp_ratio}, which should be less than 1.0");


    }
}
