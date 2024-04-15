use core::fmt;
use std::{io::{stdout, Write}, thread::sleep, time::Duration};

use anyhow::Result;
use colored::*;
use crossterm::{cursor, terminal, QueueableCommand};
use inquire::Select;
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
    println!("You now have {} in the bank", casino.bankroll);
    println!("{}", format!("* You insert your money into the {bet_amount} slot machine.").dimmed());
    sleep(Duration::from_millis(600));
    println!("{}", "* You pull the arm of the slot machine.".dimmed());
    sleep(Duration::from_millis(600));
    println!("{}", "* The wheels start spinning.".dimmed());

    let mut rng = thread_rng();

    let wheel = [
        ("🍋", 30),
        ("🍒", 30),
        ("🍊", 30),
        ("🍉", 30),
        ("🔔", 20),
        ("🍌", 20),
        ("🍫", 10),
        ("💰", 2),
        ("💎", 1),
    ];

    let symbols: Vec<&str> = wheel.iter().map(|i| i.0).collect();
    let dist = WeightedIndex::new(wheel.iter().map(|i| i.1)).unwrap();

    let mut position = 0.0;
    let mut velocity = rng.gen_range(20.0..40.0);
    let accel = rng.gen_range(-10.0..-5.0);

    let mut stdout = stdout();

    let mut samples: Vec<usize> = dist.clone().sample_iter(&mut rng).take(5).collect();
    let mut selected: Vec<&str> = samples.iter().map(|i| symbols[*i]).collect();

    while velocity > 0.0 {
        if position >= 1.0 {
            samples = dist.clone().sample_iter(&mut rng).take(5).collect();
            selected = samples.iter().map(|i| symbols[*i]).collect();
            position -= 1.0;
        }

        stdout.queue(cursor::SavePosition).unwrap();

        // stdout.write_all(" ┌──────────┐\n".as_bytes()).unwrap();
        stdout.write_all(format!("▶ {}{}{}{}{} ◀", selected[0], selected[1], selected[2], selected[3], selected[4]).as_bytes()).unwrap();
        // stdout.write_all(" └──────────┘\n".as_bytes()).unwrap();

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
    println!();

    let mut total_payout = Money::ZERO;

    for (sym, weight) in wheel.iter() {
        let count: i64 = selected.iter().filter(|s| s == &sym).count().try_into().unwrap();

        if count >= 3 {
            let sym_value = (bet_selection.multiplier * 120.0 / *weight as f32) as i64;
            let sym_payout = Money::from_major(sym_value * (count - 2));
            println!("  {} × {} = {}", sym, count, sym_payout);
            total_payout += sym_payout;
        }
    }

    println!();

    println!("Payout: {}", total_payout);

    casino.bankroll += total_payout;

    casino.save();

    Ok(())
}

pub fn symbols() -> Vec<(&'static str, u32)> {
    vec![
        ("🍋", 30),
        ("🍒", 30),
        ("🍊", 30),
        ("🍉", 30),
        ("🔔", 20),
        ("🍌", 20),
        ("🍫", 10),
        ("💰", 2),
        ("💎", 1),
    ]
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
    use rand::{distributions::{Distribution, WeightedIndex}, thread_rng};
    use rust_decimal::Decimal;

    use crate::{money::Money, slots::symbols};

    #[test]
    fn test_symbols_return_to_player() {
        let mut rng = thread_rng();
        let symbols = symbols();

        let dist = WeightedIndex::new(symbols.iter().map(|i| i.1)).unwrap();

        let mut total_player_payment = Money::ZERO;
        let mut total_player_return = Money::ZERO;

        for _i in 1..1000 {
            let samples: Vec<usize> = dist.clone().sample_iter(&mut rng).take(5).collect();
            let selected: Vec<&str> = samples.iter().map(|i| symbols[*i].0).collect();

            total_player_payment += Money::from_major(1);

            for (sym, weight) in symbols.iter() {
                let count: i64 = selected.iter().filter(|s| s == &sym).count().try_into().unwrap();

                if count >= 3 {
                    let sym_value = (120.0 / *weight as f32) as i64;
                    let sym_payout = Money::from_major(sym_value * (count - 2));
                    total_player_return += sym_payout;
                }
            }
        }

        let total_return: Decimal = total_player_return.into();
        let total_payment: Decimal = total_player_payment.into();
        let rtp_ratio: f32 = (total_return / total_payment).try_into().unwrap();

        assert!(rtp_ratio >= 0.95, "Return-to-player ratio is {rtp_ratio}, which should be higher than 0.95");
        assert!(rtp_ratio < 1.0, "Return-to-player ratio is {rtp_ratio}, which should be less than 1.0");


    }
}
