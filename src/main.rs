use std::{fs, io};
use std::path::PathBuf;
use rust_decimal::prelude::*;
use clap::Parser;
use directories::{ProjectDirs};
use serde::{Deserialize, Serialize};
use casino::cards::{Card, Hand, Value, shoe};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
  shoe_count: u8,
  shuffle_at_penetration: f32,
  #[serde(with = "rust_decimal::serde::str")]
  mister_greens_gift: Decimal,
  save_path: PathBuf,
}

impl Default for Config {
  fn default() -> Self {
    let project_dirs = ProjectDirs::from("", "", "casino").unwrap();
    let data_dir = project_dirs.data_dir();
    let save_path = data_dir.join("state.toml");

    Self {
      shoe_count: 4,
      shuffle_at_penetration: 0.75,
      mister_greens_gift: Decimal::new(1_000, 0),
      save_path: save_path,
    }
  }
}

impl Config {
  fn init_get() -> Self {
    let project_dirs = ProjectDirs::from("", "", "casino").unwrap();
    let config_dir = project_dirs.config_dir();
    fs::create_dir_all(config_dir).expect("Couldn't create config dir!");
    let config_path = config_dir.join("config.toml");

    match fs::read_to_string(&config_path) {
      Ok(config_string) => {
        toml::from_str(&config_string).unwrap()
      },
      Err(_) => {
        let config = Self::default();
        fs::write(config_path, toml::to_string(&config).unwrap()).unwrap();
        config
      }
    }
  }
}

struct Casino {
  config: Config,
  bankroll: Decimal,
  shoe: Vec<Card>,
  bet: Decimal,
  insurance_flag: bool,
}

impl Casino {
  fn new(config: Config) -> Self {
    Self {
      config: config.clone(),
      bankroll: config.mister_greens_gift,
      shoe: shoe(config.shoe_count),
      bet: Decimal::ZERO,
      insurance_flag: false,
    }
  }

  fn from_filesystem() -> Self {
    let config = Config::init_get();

    match fs::read_to_string(&config.save_path) {
      Ok(state_string) => {
        let state: CasinoState = toml::from_str(&state_string).unwrap();
        Self {
          config: config,
          bankroll: state.bankroll,
          shoe: state.shoe.clone(),
          bet: Decimal::ZERO,
          insurance_flag: false,
        }
      },
      Err(_) => {
        Self::new(config)
      }
    }
  }

  fn draw_card(&mut self) -> Card {
    let card = self.shoe.pop().unwrap();

    let threshold_fraction: f32 = 1f32 - self.config.shuffle_at_penetration;
    let starting_shoe_size: f32 = f32::from(self.config.shoe_count) * 52f32;

    let low_card_threshold: usize = (starting_shoe_size * threshold_fraction) as usize;

    if self.shoe.len() < low_card_threshold {
      self.shoe = shoe(self.config.shoe_count);
    }

    return card
  }

  fn add_bankroll(&mut self, amount: Decimal) {
    self.bankroll += amount;
  }

  fn can_initial_bet(&self, amount: Decimal) -> bool {
    amount.is_sign_positive() && !amount.is_zero() && amount <= self.bankroll
  }

  fn set_initial_bet(&mut self, amount: Decimal) {
    self.bet = amount;
  }

  fn can_bet_insurance(&self) -> bool {
    self.bet * Decimal::new(2, 0) <= self.bankroll
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

  fn blackjack_payout(&self) -> Decimal {
    self.bet * Decimal::new(15, 1).round_dp(2)
  }

  fn insurance_payout(&self) -> Decimal {
    self.bet * Decimal::new(2, 0)
  }

  fn save(&self) {
    let state = CasinoState { bankroll: self.bankroll, shoe: self.shoe.clone() };
    let save_dir = self.config.save_path.parent().unwrap();
    fs::create_dir_all(save_dir).expect("Couldn't create save directory!");
    fs::write(&self.config.save_path, toml::to_string(&state).unwrap()).unwrap();
  }
}

#[derive(Deserialize, Debug, Serialize)]
struct CasinoState {
  #[serde(with = "rust_decimal::serde::str")]
  bankroll: Decimal,
  shoe: Vec<Card>,
}

fn main() {
  let _args = Args::parse();

  let mut state = Casino::from_filesystem();

  println!("Your money: ${}", state.bankroll);

  loop {
    println!("Enter your bet: ");
    let mut bet_input = String::new();
    io::stdin().read_line(&mut bet_input).unwrap();

    let bet = bet_input.trim().parse::<Decimal>().unwrap().round_dp(2);


    if state.can_initial_bet(bet) {
      state.set_initial_bet(bet);
      break;
    } else {
      println!("You can't bet that amount, try again.");
    }
  }

  println!("Betting ${}", state.bet);

  let mut dealer_hand = Hand::new();
  dealer_hand.hidden_count = 1;
  let mut player_hand = Hand::new();

  dealer_hand.push(state.draw_card());
  player_hand.push(state.draw_card());
  dealer_hand.push(state.draw_card());
  player_hand.push(state.draw_card());

  println!("Dealer's hand: {}", dealer_hand);
  println!("Your hand: {} ({})", player_hand, player_hand.blackjack_sum());

  if matches!(dealer_hand.face_card().value, Value::Ace) && state.can_bet_insurance() {
    println!("Insurance? [y/n]");
    let mut insurance_input = String::new();
    io::stdin().read_line(&mut insurance_input).unwrap();
    match insurance_input.trim() {
      "y" => {
        state.place_insurance_bet();
        println!("You make an additional ${} insurance bet.", state.bet);
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
        player_hand.push(state.draw_card());
        println!("Your hand: {} ({})", player_hand, player_hand.blackjack_sum());

        if player_hand.blackjack_sum() > 21 {
          state.lose_bet();
          println!("BUST! You lose ${}. You now have ${}", state.bet, state.bankroll);
          break;
        }
      },
      "s" | "stand" => break,
      _ => println!("Uhh, what?"),
    }
  }

  if player_hand.blackjack_sum() <= 21 {
    dealer_hand.hidden_count = 0;
    println!("Dealer's hand: {} ({})", dealer_hand, dealer_hand.blackjack_sum());

    while dealer_hand.blackjack_sum() < 17 {
      println!("* The dealer deals themself another card");
      dealer_hand.push(state.draw_card());
      println!("Dealer's hand: {} ({})", dealer_hand, dealer_hand.blackjack_sum());
    }

    if dealer_hand.blackjack_sum() > 21 {
      state.win_bet();
      println!("DEALER BUST! You receive ${}. You now have ${}", state.bet, state.bankroll);
    } else if dealer_hand.blackjack_sum() == player_hand.blackjack_sum() {
      println!("PUSH! Nobody wins.");
    } else if dealer_hand.blackjack_sum() > player_hand.blackjack_sum() {
      state.lose_bet();
      println!("YOU LOSE! You lose ${}. You now have ${}", state.bet, state.bankroll);
    } else if player_hand.is_natural_blackjack() {
      state.win_bet_blackjack();
      let payout = state.blackjack_payout();
      println!("BLACKJACK! You receive ${payout}. You now have ${}", state.bankroll);
    } else {
      state.win_bet();
      println!("YOU WIN! You receive ${}. You now have ${}", state.bet, state.bankroll);
    }

    if dealer_hand.is_natural_blackjack() && state.insurance_flag {
      let insurance_payout = state.insurance_payout();
      state.win_insurance();
      println!("DEALER BLACKJACK! Your insurance bet pays out ${insurance_payout}. You now have ${}.", state.bankroll);
    }
  }


  if state.bankroll.is_zero() {
    state.add_bankroll(state.config.mister_greens_gift);
    println!("* Unfortunately, you've run out of money.");
    println!("* However, a portly gentleman in a sharp suit was watching you play your final hand.");
    println!("* He says \"I like your moxie, kiddo. Take this, and be a little more careful next time. This stuff doesn't grow on trees.\"");
    println!("* \"Oh, and always remember the name: MISTER GREEN!\"");
    println!("* The man hands you ${}.", state.config.mister_greens_gift);
  }

  state.save();
}


