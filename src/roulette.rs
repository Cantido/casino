use core::fmt;
use std::{io::{Write, stdout}, thread::sleep, time::Duration};

use anyhow::{ensure, Result};
use crossterm::{cursor, style, terminal, QueueableCommand};
use inquire::{Select, Text};
use num::ToPrimitive;
use rand::Rng;

use crate::{blackjack::Casino, money::Money};

pub fn play_roulette() -> Result<()> {
    let mut casino = Casino::from_filesystem()?;

    let bet = select_bet();
    let bet_amount = get_bet(&casino.bankroll);

    println!("You bet {} on {:?}", bet_amount, bet);

    let wheel = single_zero_wheel();
    let pocket = spin_wheel(wheel);

    if bet.is_match(&pocket) {
      let (num, _denom) = bet.payout();
      let payout = bet_amount * num as i64;
      println!("You win! {} has been added to your account.", payout);
      casino.add_bankroll(payout);
      println!("Your balance is now {}", casino.bankroll);
    } else {
      println!("You lose! {} has been deducted from your account.", bet_amount);
      casino.subtract_bankroll(bet_amount)?;
      println!("Your balance is now {}", casino.bankroll);
    }

    casino.save();

    Ok(())
}

fn get_bet(bankroll: &Money) -> Money {
    println!("Your money: {}", bankroll);

    loop {
      let bet_result = Text::new("How much will you bet?").prompt();

      match bet_result {
        Ok(bet_text) => {
          let bet = bet_text.trim().parse::<Money>().unwrap();
          if bet < *bankroll {
            return bet;
          } else {
            println!("You can't bet that amount, try again.");
          }
        },
        Err(_) => panic!("Error getting your answer."),
      }
    }
}

fn select_bet() -> RouletteBet {
    loop {
      let options: Vec<RouletteBetType> = vec![
        RouletteBetType::Straight,
        RouletteBetType::Color,
        RouletteBetType::Column,
        RouletteBetType::Dozens,
        RouletteBetType::HighsLows,
        RouletteBetType::OddsEvens
      ];
      let selected = Select::new("What type of bet will you make?", options).prompt();

      match selected {
        Ok(RouletteBetType::Straight) => {
          return Select::new("Which number will you bet on?", RouletteBetType::Straight.bets()).prompt().unwrap();
        },
        Ok(RouletteBetType::Color) => {
          return Select::new("Which color will you bet on?", RouletteBetType::Color.bets()).prompt().unwrap();
        },
        Ok(RouletteBetType::Dozens) => {
          return Select::new("Which dozen will you bet on?", RouletteBetType::Dozens.bets()).prompt().unwrap();
        },
        Ok(RouletteBetType::HighsLows) => {
          return Select::new("Will you bet on highs or lows?", RouletteBetType::HighsLows.bets()).prompt().unwrap();
        },
        Ok(RouletteBetType::OddsEvens) => {
          return Select::new("Will you bet on odds or evens?", RouletteBetType::OddsEvens.bets()).prompt().unwrap();
        }
        Ok(RouletteBetType::Column) => {
          return Select::new("Which column will you bet on?", RouletteBetType::Column.bets()).prompt().unwrap();
        }
        Err(_) => panic!("Error getting your answer."),
      }
    }
}

fn spin_wheel(wheel: Vec<Pocket>) -> Pocket {
    let mut rng = rand::thread_rng();
    let mut position = rng.gen_range(0.0..37.0);
    let mut velocity = rng.gen_range(20.0..40.0);
    let accel = rng.gen_range(-10.0..-5.0);

    let mut stdout = stdout();

    while velocity > 0.0 {
      let index = position.to_usize().unwrap();
      let pocket = &wheel[index];

      let bgcolor = match pocket.color {
        Color::Green => style::Color::DarkGreen,
        Color::Black => style::Color::Black,
        Color::Red => style::Color::DarkRed,
      };

      stdout.queue(cursor::SavePosition).unwrap();
      stdout.queue(style::SetBackgroundColor(bgcolor)).unwrap();
      stdout.queue(style::SetForegroundColor(style::Color::White)).unwrap();
      stdout.write_all(format!("{}", pocket.value).as_bytes()).unwrap();
      stdout.queue(cursor::RestorePosition).unwrap();
      stdout.flush().unwrap();

      sleep(Duration::from_millis(16));

      velocity += accel * (16.0 / 1000.0);
      position += velocity * (16.0 / 1000.0);

      position = position % 37.0;

      stdout.queue(cursor::RestorePosition).unwrap();
      stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
    }

    let index = position.to_usize().unwrap();
    let pocket = &wheel[index];
    println!("{} ({:?})", pocket.value, pocket.color);

    sleep(Duration::from_millis(1200));

    pocket.clone()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Color {
  Black,
  Green,
  Red,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Parity {
  Odd,
  Even,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum HighLow {
  High,
  Low,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Dozen {
  First,
  Second,
  Third,
}

#[derive(Clone, Debug)]
struct Pocket {
  value: u8,
  color: Color,
}

impl Pocket {
  fn new(val: u8) -> Result<Self> {
    ensure!(val <= 36, "Number {} is outside of the range 0..=36", val);

    if val == 0 {
      Ok(Pocket { value: val, color: Color::Green })
    } else if (val >= 1 && val <= 10) || (val >= 19 && val <= 28) {
      let color =
        if val % 2 == 0 {
          Color::Black
        } else {
          Color::Red
        };

      Ok(Pocket { value: val, color })
    } else {
      let color =
        if val % 2 == 0 {
          Color::Red
        } else {
          Color::Black
        };

      Ok(Pocket { value: val, color })
    }
  }
}

fn single_zero_wheel() -> Vec<Pocket> {
  vec![
    0, 32, 15, 19, 4, 21, 2, 25, 17,
    34, 6, 27, 3, 36, 11, 30, 8, 23,
    10, 5, 24, 16, 33, 1, 0, 14, 31,
    9, 22, 18, 29, 7, 27, 12, 35, 3, 26
  ].into_iter().map(|i| Pocket::new(i).unwrap()).collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RouletteBetType {
  Straight,
  Color,
  Dozens,
  HighsLows,
  OddsEvens,
  Column,
}

impl RouletteBetType {
  fn bets(&self) -> Vec<RouletteBet> {
    match self {
      RouletteBetType::Straight => (0..=36).into_iter().map(|v| RouletteBet::Straight(v)).collect(),
      RouletteBetType::Color => vec![RouletteBet::Color(Color::Red), RouletteBet::Color(Color::Black)],
      RouletteBetType::Dozens => vec![
        RouletteBet::Dozens(Dozen::First),
        RouletteBet::Dozens(Dozen::Second),
        RouletteBet::Dozens(Dozen::Third),
      ],
      RouletteBetType::Column => {
        vec![
          RouletteBet::Column(vec![1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 34]),
          RouletteBet::Column(vec![2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35]),
          RouletteBet::Column(vec![3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36]),
        ]
      },
      RouletteBetType::HighsLows => vec![RouletteBet::HighsLows(HighLow::High), RouletteBet::HighsLows(HighLow::Low)],
      RouletteBetType::OddsEvens => vec![RouletteBet::OddsEvens(Parity::Odd), RouletteBet::OddsEvens(Parity::Even)],
    }
  }
}

impl fmt::Display for RouletteBetType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
        RouletteBetType::Straight => write!(f, "straight"),
        RouletteBetType::Color => write!(f, "color"),
        RouletteBetType::Dozens => write!(f, "dozen"),
        RouletteBetType::Column => write!(f, "column"),
        RouletteBetType::HighsLows => write!(f, "highs/lows"),
        RouletteBetType::OddsEvens => write!(f, "odds/evens"),
      }
  }
}

#[derive(Debug)]
enum RouletteBet {
  Straight(u8),
  Color(Color),
  Dozens(Dozen),
  HighsLows(HighLow),
  OddsEvens(Parity),
  Column(Vec<u8>),
}

impl RouletteBet {
  fn is_match(&self, pocket: &Pocket) -> bool {
    match self {
      RouletteBet::Straight(val) => pocket.value == *val,
      RouletteBet::Color(color) => pocket.color == *color,
      RouletteBet::Dozens(Dozen::First) => pocket.value >= 1 && pocket.value <= 12,
      RouletteBet::Dozens(Dozen::Second) => pocket.value >= 13 && pocket.value <= 24,
      RouletteBet::Dozens(Dozen::Third) => pocket.value >= 25 && pocket.value <= 36,
      RouletteBet::HighsLows(HighLow::Low) => pocket.value >= 1 && pocket.value <= 18,
      RouletteBet::HighsLows(HighLow::High) => pocket.value >= 19 && pocket.value <= 36,
      RouletteBet::OddsEvens(Parity::Odd) => pocket.value % 2 == 1,
      RouletteBet::OddsEvens(Parity::Even) => pocket.value % 2 == 0,
      RouletteBet::Column(vals) => vals.contains(&pocket.value),
    }
  }

  fn payout(&self) -> (u8, u8) {
    match self {
      RouletteBet::Straight(_) => (35, 1),
      RouletteBet::Color(_) => (1, 1),
      RouletteBet::Dozens(_) => (2, 1),
      RouletteBet::HighsLows(_) => (1, 1),
      RouletteBet::OddsEvens(_) => (1, 1),
      RouletteBet::Column(_) => (2, 1),
    }
  }
}

impl fmt::Display for RouletteBet {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
        RouletteBet::Straight(val) => {
          let pocket = Pocket::new(*val).unwrap();
          write!(f, "{} ({:?})", pocket.value, pocket.color)
        },
        RouletteBet::Color(col) => {
          write!(f, "{:?}", col)
        },
        RouletteBet::Dozens(dozen) => {
          write!(f, "{:?}", dozen)
        },
        RouletteBet::Column(vals) => {
          let strings: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
          write!(f, "{}", strings.join(", "))
        },
        RouletteBet::HighsLows(side) => {
          write!(f, "{:?}", side)
        },
        RouletteBet::OddsEvens(parity) => {
          write!(f, "{:?}", parity)
        }
      }
  }
}
