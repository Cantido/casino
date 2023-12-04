use std::{fmt, fs, fs::OpenOptions, io, io::prelude::*, io::Read, io::SeekFrom, io::Write};
use std::path::Path;
use rust_decimal::prelude::*;
use clap::Parser;
use directories::{ProjectDirs};
use serde::{Deserialize, Serialize};
use casino::cards::{Card, Value, shoe};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

}

#[derive(Deserialize, Debug, Serialize)]
struct CasinoState {
  bankroll: u32,
  shoe: Vec<Card>,
}

struct Casino {
  bankroll: u32,
  shoe: Vec<Card>,
  bet: u32,
  insurance_flag: bool,
}

impl Casino {
  fn new() -> Self {
    Casino {
      bankroll: 1_000,
      shoe: shoe(4),
      bet: 0,
      insurance_flag: false,
    }
  }

  fn from_path(path: &Path) -> Self {
    match fs::read_to_string(&path) {
      Ok(state_string) => {
        let state: CasinoState = toml::from_str(&state_string).unwrap();
        Self {
          bankroll: state.bankroll,
          shoe: state.shoe.clone(),
          bet: 0,
          insurance_flag: false,
        }
      },
      Err(_) => {
        Self::new()
      }
    }
  }

  fn draw_card(&mut self) -> Card {
    let card = self.shoe.pop().unwrap();

    // assuming 4 decks in the shoe
    if self.shoe.len() < 52 {
      self.shoe = shoe(4);
    }

    return card
  }

  fn add_bankroll(&mut self, amount: u32) {
    self.bankroll += amount;
  }

  fn can_initial_bet(&self, amount: u32) -> bool {
    amount > 0 && amount <= self.bankroll
  }

  fn set_initial_bet(&mut self, amount: u32) {
    self.bet = amount;
  }

  fn can_bet_insurance(&self) -> bool {
    self.bet * 2 <= self.bankroll
  }

  fn place_insurance_bet(&mut self) {
    self.insurance_flag = true;
  }

  fn lose_bet(&mut self) {
    self.bankroll -= self.bet;
  }

  fn win_bet(&mut self) {
    self.bankroll += self.bet;
  }

  fn win_bet_blackjack(&mut self) {
    self.bankroll += self.blackjack_payout();
  }

  fn win_insurance(&mut self) {
    self.bankroll += self.insurance_payout();
  }

  fn blackjack_payout(&self) -> u32 {
    self.bet * 3 / 2
  }

  fn insurance_payout(&self) -> u32 {
    self.bet * 2
  }

  fn persist(&self, path: &Path) {
    let state = CasinoState { bankroll: self.bankroll, shoe: self.shoe.clone() };
    fs::write(path, toml::to_string(&state).unwrap());
  }
}

fn main() {
  let args = Args::parse();

  let project_dirs = ProjectDirs::from("", "", "casino").unwrap();
  let data_dir = project_dirs.data_dir();

  fs::create_dir_all(data_dir).expect("Couldn't create data dir!");

  let state_path = data_dir.join("state");

  let mut state = Casino::from_path(&state_path);

  println!("Your money: ${}.00", state.bankroll);

  loop {
    println!("Enter your bet: ");
    let mut bet_input = String::new();
    io::stdin().read_line(&mut bet_input).unwrap();

    let bet = bet_input.trim().parse().unwrap();

    if state.can_initial_bet(bet) {
      state.set_initial_bet(bet);
      break;
    } else {
      println!("You can't bet that amount, try again.");
    }
  }

  println!("Betting ${}.00", state.bet);

  let mut dealer_hidden;
  let mut dealer_shown;
  let mut your_hand = vec![];

  dealer_hidden = state.draw_card();
  your_hand.push(state.draw_card());
  dealer_shown = state.draw_card();
  your_hand.push(state.draw_card());

  println!("Dealer's hand: 🂠 {}", dealer_shown);
  println!("Your hand: {} ({})", hand_to_string(&your_hand), blackjack_sum(&your_hand));

  let mut insurance_flag = false;

  if matches!(dealer_shown.value, Value::Ace) && state.can_bet_insurance() {
    println!("Insurance? [y/n]");
    let mut insurance_input = String::new();
    io::stdin().read_line(&mut insurance_input).unwrap();
    match insurance_input.trim() {
      "y" => {
        state.place_insurance_bet();
        println!("You make an additional ${}.00 insurance bet.", state.bet);
      }
      _ => {
        println!("You choose for forgo making an insurance bet.");
      }
    }
  }

  loop {
    println!("Hit or stand? [h/s]:");

    let mut hit_stand_input = String::new();
    io::stdin().read_line(&mut hit_stand_input).unwrap();
    match hit_stand_input.trim() {
      "h" | "hit" => {
        println!("* The dealer deals you another card");
        your_hand.push(state.draw_card());
        let sum = blackjack_sum(&your_hand);
        println!("Your hand: {} ({})", hand_to_string(&your_hand), sum);

        if sum > 21 {
          state.lose_bet();
          println!("BUST! You lose ${}.00. You now have ${}.00", state.bet, state.bankroll);
          break;
        }
      },
      "s" | "stand" => break,
      _ => println!("Uhh, what?"),
    }
  }

  let player_sum = blackjack_sum(&your_hand);

  if player_sum <= 21 {
    let mut dealer_hand = vec![dealer_hidden, dealer_shown];
    let mut dealer_sum = blackjack_sum(&dealer_hand);
    println!("Dealer's hand: {} ({})", hand_to_string(&dealer_hand), dealer_sum);

    while dealer_sum < 17 {
      println!("* The dealer deals themself another card");
      dealer_hand.push(state.draw_card());
      dealer_sum = blackjack_sum(&dealer_hand);
      println!("Dealer's hand: {} ({})", hand_to_string(&dealer_hand), dealer_sum);
    }

    if dealer_sum > 21 {
      state.win_bet();
      println!("DEALER BUST! You receive ${}.00. You now have ${}.00", state.bet, state.bankroll);
    } else if dealer_sum == player_sum {
      println!("PUSH! Nobody wins.");
    } else if dealer_sum > player_sum {
      state.lose_bet();
      println!("YOU LOSE! You lose ${}.00. You now have ${}.00", state.bet, state.bankroll);
    } else if your_hand.len() == 2 && player_sum == 21 {
      state.win_bet_blackjack();
      let payout = state.blackjack_payout();
      println!("BLACKJACK! You receive ${payout}.00. You now have ${}.00", state.bankroll);
    } else {
      state.win_bet();
      println!("YOU WIN! You receive ${}. You now have ${}.00", state.bet, state.bankroll);
    }

    if dealer_hand.len() == 2 && dealer_sum == 21 && insurance_flag {
      let insurance_payout = state.insurance_payout();
      state.win_insurance();
      println!("DEALER BLACKJACK! Your insurance bet pays out ${insurance_payout}.00. You now have ${}.00.", state.bankroll);
    }
  }


  if state.bankroll.is_zero() {
    state.add_bankroll(1_000);
    println!("* Unfortunately, you've run out of money.");
    println!("* However, a portly gentleman in a sharp suit was watching you play your final hand.");
    println!("* He says \"I like your moxie, kiddo. Take this, and be a little more careful next time. This stuff doesn't grow on trees.\"");
    println!("* \"Oh, and always remember the name: MISTER GREEN!\"");
    println!("* The man hands you $1000.00.");
  }

  state.persist(&state_path);
}

fn blackjack_sum(hand: &Vec<Card>) -> u8 {
  let mut sum = 0;
  for card in hand.iter() {
    sum += card.blackjack_value();
  }

  let has_ace = hand.iter().any(|c| matches!(&c.value, Value::Ace));

  if has_ace && sum <= 11 {
    sum += 10;
  }

  return sum
}

fn hand_to_string(hand: &Vec<Card>) -> String {
  let mut hand_str: String = "".to_owned();

  for card in hand.iter() {
    hand_str.push_str(&format!("{}", card));
  }

  return hand_str
}
