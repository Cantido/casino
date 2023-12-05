use std::fs;
use std::path::PathBuf;
use inquire::{Confirm, Select, Text};
use rust_decimal::prelude::*;
use clap::{Parser, Subcommand};
use directories::{ProjectDirs};
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::thread::sleep;
use std::time::Duration;
use casino::cards::{Card, Hand, Value, shoe};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  command: Option<Commands>
}

#[derive(Debug, Subcommand)]
enum Commands {
  /// Play a hand of blackjack
  Blackjack,
  /// Show lifetime statistics
  Stats,
  /// Shuffle the persisted deck state
  Shuffle,
  /// Show currency balance
  Balance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
  #[serde(default = "Config::default_shoe_count")]
  shoe_count: u8,
  #[serde(default = "Config::default_shuffle_penetration")]
  shuffle_at_penetration: f32,
  #[serde(with = "rust_decimal::serde::str")]
  #[serde(default = "Config::default_greens_gift")]
  mister_greens_gift: Decimal,
  #[serde(default = "Config::default_save_path")]
  save_path: PathBuf,
  #[serde(default = "Config::default_stats_path")]
  stats_path: PathBuf,
}

impl Config {
  fn default_shoe_count() -> u8 {
    4
  }

  fn default_shuffle_penetration() -> f32 {
    0.75
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
      shoe_count: Self::default_shoe_count(),
      shuffle_at_penetration: Self::default_shuffle_penetration(),
      mister_greens_gift: Self::default_greens_gift(),
      save_path: Self::default_save_path(),
      stats_path: Self::default_stats_path(),
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
        match toml::from_str(&config_string) {
          Ok(config) => config,
          Err(_) => {
            let _ = fs::remove_file(&config_path);
            let config = Self::default();
            fs::write(config_path, toml::to_string(&config).unwrap()).unwrap();
            config
          }
        }
      },
      Err(_) => {
        let config = Self::default();
        fs::write(config_path, toml::to_string(&config).unwrap()).unwrap();
        config
      }
    }
  }
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct Statistics {
  hands_won: u32,
  hands_lost: u32,
  hands_push: u32,
  #[serde(with = "rust_decimal::serde::str")]
  money_won: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  money_lost: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  biggest_win: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  biggest_loss: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  biggest_bankroll: Decimal,
  times_bankrupted: u32,
}

impl Statistics {
  fn record_win(&mut self, amount: Decimal) {
    self.hands_won += 1;
    self.money_won += amount;
    if amount > self.biggest_win {
      self.biggest_win = amount;
    }
  }

  fn record_loss(&mut self, amount: Decimal) {
    self.hands_lost += 1;
    self.money_lost += amount;
    if amount > self.biggest_loss {
      self.biggest_loss = amount;
    }
  }

  fn record_push(&mut self) {
    self.hands_push += 1;
  }

  fn update_bankroll(&mut self, amount: Decimal) {
    if amount > self.biggest_bankroll {
      self.biggest_bankroll = amount;
    } else if amount.is_zero() {
      self.times_bankrupted += 1;
    }
  }
}

struct Casino {
  config: Config,
  bankroll: Decimal,
  shoe: Vec<Card>,
  bet: Decimal,
  insurance_bet: Decimal,
  split_bet: Decimal,
  standing: bool,
  standing_split: bool,
  doubling_down: bool,
  splitting: bool,
  stats: Statistics,
  dealer_hand: Hand,
  player_hand: Hand,
  split_hand: Hand,
}

impl Casino {
  fn new(config: Config, stats: Statistics) -> Self {
    Self {
      config: config.clone(),
      stats: stats.clone(),
      bankroll: config.mister_greens_gift,
      shoe: shoe(config.shoe_count),
      bet: Decimal::ZERO,
      insurance_bet: Decimal::ZERO,
      split_bet: Decimal::ZERO,
      standing: false,
      standing_split: false,
      doubling_down: false,
      splitting: false,
      dealer_hand: Hand::new(),
      player_hand: Hand::new(),
      split_hand: Hand::new(),
    }
  }

  fn from_filesystem() -> Self {
    let config = Config::init_get();

    let stats: Statistics = match fs::read_to_string(&config.stats_path) {
      Ok(stats_string) => {
        toml::from_str(&stats_string).unwrap()
      },
      Err(_) => {
        Statistics::default()
      }
    };

    match fs::read_to_string(&config.save_path) {
      Ok(state_string) => {
        let state: CasinoState = toml::from_str(&state_string).unwrap();
        Self {
          config: config,
          stats: stats,
          bankroll: state.bankroll,
          shoe: state.shoe.clone(),
          bet: Decimal::ZERO,
          insurance_bet: Decimal::ZERO,
          split_bet: Decimal::ZERO,
          standing: false,
          standing_split: false,
          doubling_down: false,
          splitting: false,
          dealer_hand: Hand::new_hidden(1),
          player_hand: Hand::new(),
          split_hand: Hand::new(),
        }
      },
      Err(_) => {
        Self::new(config, stats)
      }
    }
  }

  fn draw_card(&mut self) -> Card {
    let card = self.shoe.pop().unwrap();

    let threshold_fraction: f32 = 1f32 - self.config.shuffle_at_penetration;
    let starting_shoe_size: f32 = f32::from(self.config.shoe_count) * 52f32;

    let low_card_threshold: usize = (starting_shoe_size * threshold_fraction) as usize;

    if self.shoe.len() < low_card_threshold {
      self.shuffle_shoe();
    }

    return card
  }

  fn shuffle_shoe(&mut self) {
    self.shoe = shoe(self.config.shoe_count);
  }

  fn card_to_dealer(&mut self) {
    let card = self.draw_card();
    self.dealer_hand.push(card);
  }

  fn card_to_player(&mut self) {
    let card = self.draw_card();
    self.player_hand.push(card);
  }

  fn card_to_split(&mut self) {
    let card = self.draw_card();
    self.split_hand.push(card);
  }

  fn add_bankroll(&mut self, amount: Decimal) {
    self.bankroll += amount;
    self.stats.update_bankroll(self.bankroll);
  }

  fn can_increase_bet(&self, amount: Decimal) -> bool {
    amount.is_sign_positive() && !amount.is_zero() && amount <= self.bankroll
  }

  fn increase_bet(&mut self, amount: Decimal) {
    self.bet += amount;
    self.bankroll -= amount;
  }

  fn can_place_insurance_bet(&self) -> bool {
    match self.dealer_hand.face_card().value {
      Value::Ace => self.bet <= self.bankroll,
      _ => false,
    }
  }

  fn place_insurance_bet(&mut self) {
    let bet_amount = (self.bet / Decimal::new(2, 0)).round_dp(2);
    self.insurance_bet += bet_amount;
    self.bankroll -= bet_amount;
  }

  fn can_double_down(&self) -> bool {
    let player_sum = self.player_hand.blackjack_sum();
    self.player_hand.cards.len() == 2 &&
      !self.doubling_down &&
      self.bet <= self.bankroll &&
      (player_sum == 10 || player_sum == 11)
  }

  fn double_down(&mut self) {
    self.doubling_down = true;
    self.bet *= Decimal::new(2, 0);
  }

  fn can_split(&self) -> bool {
    !self.splitting &&
    self.can_increase_bet(self.bet) &&
    self.player_hand.cards.len() == 2 &&
    self.player_hand.cards[0].value == self.player_hand.cards[1].value
  }

  fn split(&mut self) {
    let moved_card = self.player_hand.cards.pop().unwrap();
    self.split_hand.push(moved_card);

    self.splitting = true;
    self.split_bet += self.bet;
    self.bankroll -= self.bet;
  }

  fn lose_bet(&mut self) {
    self.stats.record_loss(self.bet);
    self.stats.update_bankroll(self.bankroll);
    self.bet = Decimal::ZERO;
  }

  fn lose_split_bet(&mut self) {
    self.stats.record_loss(self.split_bet);
    self.stats.update_bankroll(self.bankroll);
    self.split_bet = Decimal::ZERO;
  }

  fn win_bet(&mut self) {
    self.stats.record_win(self.win_payout());
    self.bankroll += self.bet + self.win_payout();
    self.stats.update_bankroll(self.bankroll);
    self.bet = Decimal::ZERO;
  }

  fn win_split_bet(&mut self) {
    self.stats.record_win(self.win_split_payout());
    self.bankroll += self.split_bet + self.win_split_payout();
    self.stats.update_bankroll(self.bankroll);
    self.split_bet = Decimal::ZERO;
  }

  fn win_bet_blackjack(&mut self) {
    self.stats.record_win(self.blackjack_payout());
    self.bankroll += self.bet + self.blackjack_payout();
    self.stats.update_bankroll(self.bankroll);
    self.bet = Decimal::ZERO;
  }
  fn win_split_bet_blackjack(&mut self) {
    self.stats.record_win(self.split_blackjack_payout());
    self.bankroll += self.split_bet + self.split_blackjack_payout();
    self.stats.update_bankroll(self.bankroll);
    self.split_bet = Decimal::ZERO;
  }

  fn win_insurance(&mut self) {
    self.bankroll += self.bet + self.insurance_payout();
    self.stats.update_bankroll(self.bankroll);
    self.insurance_bet = Decimal::ZERO;
  }

  fn push_bet(&mut self) {
    self.stats.record_push();
    self.bankroll += self.bet;
    self.bet = Decimal::ZERO;
  }

  fn push_split_bet(&mut self) {
    self.stats.record_push();
    self.bankroll += self.split_bet;
    self.split_bet = Decimal::ZERO;
  }

  fn win_payout(&self) -> Decimal {
    self.bet
  }

  fn win_split_payout(&self) -> Decimal {
    self.split_bet
  }

  fn blackjack_payout(&self) -> Decimal {
    self.bet * Decimal::new(15, 1).round_dp(2)
  }

  fn split_blackjack_payout(&self) -> Decimal {
    self.split_bet * Decimal::new(15, 1).round_dp(2)
  }

  fn insurance_payout(&self) -> Decimal {
    self.insurance_bet * Decimal::new(2, 0)
  }

  fn save(&self) {
    let state = CasinoState { bankroll: self.bankroll, shoe: self.shoe.clone() };
    let save_dir = self.config.save_path.parent().unwrap();
    fs::create_dir_all(save_dir).expect("Couldn't create save directory!");
    fs::write(&self.config.save_path, toml::to_string(&state).unwrap()).unwrap();

    let stats_dir = self.config.stats_path.parent().unwrap();
    fs::create_dir_all(stats_dir).expect("Couldn't create stats directory!");
    fs::write(&self.config.stats_path, toml::to_string(&self.stats).unwrap()).unwrap();
  }
}

#[derive(Deserialize, Debug, Serialize)]
struct CasinoState {
  #[serde(with = "rust_decimal::serde::str")]
  bankroll: Decimal,
  shoe: Vec<Card>,
}

fn main() {
  let args = Args::parse();

  match &args.command {
    Some(Commands::Stats) => {
      let state = Casino::from_filesystem();
      let stats = state.stats;

      println!("Hands won...............{:.>15}", stats.hands_won);
      println!("Hands lost..............{:.>15}", stats.hands_lost);
      println!("Hands tied..............{:.>15}", stats.hands_push);
      println!("Times hit bankruptcy....{:.>15}", stats.times_bankrupted);
      println!("Total money won.........{:.>15.2}", stats.money_won);
      println!("Total money lost........{:.>15.2}", stats.money_lost);
      println!("Biggest win.............{:.>15.2}", stats.biggest_win);
      println!("Biggest loss............{:.>15.2}", stats.biggest_loss);
      println!("Most money in the bank..{:.>15.2}", stats.biggest_bankroll);
    },
    Some(Commands::Blackjack) => {
      play_blackjack();
    },
    Some(Commands::Shuffle) => {
      let mut state = Casino::from_filesystem();
      state.shuffle_shoe();
      state.save();
    },
    Some(Commands::Balance) => {
      let state = Casino::from_filesystem();
      println!("${:.2}", state.bankroll);
    }
    None => {
      let options = vec!["Blackjack"];

      let ans = Select::new("What would you like to play?", options).prompt();

      match ans {
        Ok("Blackjack") => play_blackjack(),
        Ok(_) => panic!("Unknown option"),
        Err(_) => panic!("Error fetching your choice"),
      }
    }
  }
}

fn play_blackjack() {
  let mut state = Casino::from_filesystem();

  println!("Your money: ${:.2}", state.bankroll);

  loop {
    let bet_result = Text::new("How much will you bet?").prompt();

    match bet_result {
      Ok(bet_text) => {
        let bet = bet_text.trim().parse::<Decimal>().unwrap().round_dp(2);
        if state.can_increase_bet(bet) {
          state.increase_bet(bet);
          break;
        } else {
          println!("You can't bet that amount, try again.");
        }
      },
      Err(_) => panic!("Error getting your answer."),
    }
  }

  println!("Betting ${:.2}", state.bet);

  let mut sp = Spinner::new(Spinners::Dots, "Dealing cards...".into());
  sleep(Duration::from_millis(1_500));
  sp.stop_with_message("* The dealer issues your cards.".into());

  state.card_to_dealer();
  state.card_to_player();
  state.card_to_dealer();
  state.card_to_player();

  println!("Dealer's hand: {}", state.dealer_hand);
  println!("Your hand: {} ({})", state.player_hand, state.player_hand.blackjack_sum());

  if state.can_place_insurance_bet() {
    let ans = Confirm::new("Insurance?").with_default(false).prompt();

    match ans {
      Ok(true) => {
        state.place_insurance_bet();
        println!("You make an additional ${:.2} insurance bet.", state.insurance_bet);
      },
      Ok(false) => println!("You choose for forgo making an insurance bet."),
      Err(_) => panic!("Error getting your answer"),
    }
  }

  let mut current_hand = 0;

  while !(state.standing || state.player_hand.blackjack_sum() > 21) || (state.splitting && !(state.standing_split || state.split_hand.blackjack_sum() > 21)) {

    let mut options = vec!["Hit", "Stand"];

    if state.can_double_down() {
      options.push("Double");
    }

    if state.can_split() {
      options.push("Split");
    }

    let prompt =
      if state.splitting && current_hand == 0 {
        "What will you do with your first hand?"
      } else if state.splitting && current_hand == 1 {
        "What will you do with your second hand?"
      } else {
        "What will you do?"
      };

    let ans = Select::new(prompt, options).prompt();

    match ans {
      Ok("Hit") => {
        let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
        sleep(Duration::from_millis(1_000));
        sp.stop_with_message("* The dealer hands you another card.".into());

        if state.splitting && current_hand == 0 {
          state.card_to_player();
          println!("Your first hand: {} ({})", state.player_hand, state.player_hand.blackjack_sum());

          if state.player_hand.blackjack_sum() > 21 {
            let bet = state.bet;
            state.lose_bet();
            current_hand = 1;
            println!("FIRST HAND BUST! You lose ${:.2}. You now have ${:.2}", bet, state.bankroll);
          }
        } else if state.splitting && current_hand == 1 {
          state.card_to_split();
          println!("Your second hand: {} ({})", state.split_hand, state.split_hand.blackjack_sum());

          if state.split_hand.blackjack_sum() > 21 {
            let bet = state.split_bet;
            state.lose_split_bet();
            println!("SECOND HAND BUST! You lose ${:.2}. You now have ${:.2}", bet, state.bankroll);
          }
        } else {
          state.card_to_player();
          println!("Your hand: {} ({})", state.player_hand, state.player_hand.blackjack_sum());

          if state.player_hand.blackjack_sum() > 21 {
            let bet = state.bet;
            state.lose_bet();
            println!("BUST! You lose ${:.2}. You now have ${:.2}", bet, state.bankroll);
          }
        }
      },
      Ok("Double") => {
        state.double_down();
        println!("Your bet is now ${:.2}, and you will only receive one more card.", state.bet);

        let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
        sleep(Duration::from_millis(1_000));
        sp.stop_with_message("* The dealer hands you another card.".into());

        state.card_to_player();
        println!("Your hand: {} ({})", state.player_hand, state.player_hand.blackjack_sum());

        if state.player_hand.blackjack_sum() > 21 {
          let bet = state.bet;
          state.lose_bet();
          println!("BUST! You lose ${:.2}. You now have ${:.2}", bet, state.bankroll);
        }
        state.standing = true;
      },
      Ok("Split") => {
        state.split();
        println!("You split your hand and place a second ${:.2} bet.", state.split_bet);

        let mut sp = Spinner::new(Spinners::Dots, "Dealing your cards...".into());
        sleep(Duration::from_millis(1_000));
        sp.stop_with_message("* The dealer hands you another two cards.".into());

        state.card_to_player();
        state.card_to_split();

        println!("Your first hand: {} ({})", state.player_hand, state.player_hand.blackjack_sum());
        println!("Your second hand: {} ({})", state.split_hand, state.split_hand.blackjack_sum());

        if state.player_hand.blackjack_sum() > 21 {
          let bet = state.bet;
          state.lose_bet();
          println!("FIRST HAND BUST! You lose ${:.2}. You now have ${:.2}", bet, state.bankroll);
        }

        if state.split_hand.blackjack_sum() > 21 {
          let bet = state.split_bet;
          state.lose_split_bet();
          println!("SECOND HAND BUST! You lose ${:.2}. You now have ${:.2}", bet, state.bankroll);
        }
      },
      Ok("Stand") => {
        if current_hand == 0 {
          state.standing = true;
          current_hand = 1;
        } else if current_hand == 1 {
          state.standing_split = true;
        }
      },
      Ok(_) => panic!("Unknown answer received"),
      Err(_) => panic!("Error getting your answer."),
    }
  }


  if state.player_hand.blackjack_sum() <= 21 || (state.splitting && state.split_hand.blackjack_sum() <= 21) {
    let mut sp = Spinner::new(Spinners::Dots, "Revealing the hole card...".into());
    sleep(Duration::from_millis(1_000));
    sp.stop_with_message("* Hole card revealed!".into());

    state.dealer_hand.hidden_count = 0;
    println!("Dealer's hand: {} ({})", state.dealer_hand, state.dealer_hand.blackjack_sum());

    while state.dealer_hand.blackjack_sum() < 17 {
      let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
      sleep(Duration::from_millis(1_000));
      sp.stop_with_message("* The dealer issues themself another card.".into());

      state.card_to_dealer();
      println!("Dealer's hand: {} ({})", state.dealer_hand, state.dealer_hand.blackjack_sum());
    }

    let mut sp = Spinner::new(Spinners::Dots, "Determining outcome...".into());
    sleep(Duration::from_millis(1_000));
    sp.stop_with_message("* The hand is finished!".into());

    if state.player_hand.blackjack_sum() <= 21 {
      if state.splitting {
        print!("First hand result: ");
      }

      if state.dealer_hand.blackjack_sum() > 21 {
        let bet = state.bet;
        state.win_bet();
        println!("DEALER BUST! You receive ${:.2}. You now have ${:.2}", bet, state.bankroll);
      } else if state.dealer_hand.blackjack_sum() == state.player_hand.blackjack_sum() {
        state.push_bet();
        println!("PUSH! Nobody wins.");
      } else if state.dealer_hand.blackjack_sum() > state.player_hand.blackjack_sum() {
        let bet = state.bet;
        state.lose_bet();
        println!("HOUSE WINS! You lose ${:.2}. You now have ${:.2}", bet, state.bankroll);
      } else if state.player_hand.is_natural_blackjack() {
        state.win_bet_blackjack();
        let payout = state.blackjack_payout();
        println!("BLACKJACK! You receive ${:.2}. You now have ${:.2}", payout, state.bankroll);
      } else {
        let bet = state.bet;
        state.win_bet();
        println!("YOU WIN! You receive ${:.2}. You now have ${:.2}", bet, state.bankroll);
      }
    }

    if state.splitting && state.split_hand.blackjack_sum() <= 21 {
      print!("Second hand result: ");

      if state.dealer_hand.blackjack_sum() > 21 {
        let bet = state.split_bet;
        state.win_split_bet();
        println!("DEALER BUST! You receive ${:.2}. You now have ${:.2}", bet, state.bankroll);
      } else if state.dealer_hand.blackjack_sum() == state.split_hand.blackjack_sum() {
        state.push_split_bet();
        println!("PUSH! Nobody wins.");
      } else if state.dealer_hand.blackjack_sum() > state.split_hand.blackjack_sum() {
        let bet = state.split_bet;
        state.lose_split_bet();
        println!("HOUSE WINS! You lose ${:.2}. You now have ${:.2}", bet, state.bankroll);
      } else if state.split_hand.is_natural_blackjack() {
        state.win_split_bet_blackjack();
        let payout = state.split_blackjack_payout();
        println!("BLACKJACK! You receive ${:.2}. You now have ${:.2}", payout, state.bankroll);
      } else {
        let bet = state.split_bet;
        state.win_split_bet();
        println!("YOU WIN! You receive ${:.2}. You now have ${:.2}", bet, state.bankroll);
      }
    }

    if state.dealer_hand.is_natural_blackjack() && !state.insurance_bet.is_zero() {
      let insurance_payout = state.insurance_payout();
      state.win_insurance();
      println!("DEALER BLACKJACK! Your insurance bet pays out ${:.2}. You now have ${:.2}.", insurance_payout, state.bankroll);
    }
  }


  if state.bankroll.is_zero() {
    state.add_bankroll(state.config.mister_greens_gift);
    println!("* Unfortunately, you've run out of money.");
    println!("* However, a portly gentleman in a sharp suit was watching you play your final hand.");
    println!("* He says \"I like your moxie, kiddo. Take this, and be a little more careful next time. This stuff doesn't grow on trees.\"");
    println!("* \"Oh, and always remember the name: MISTER GREEN!\"");
    println!("* The man hands you ${:.2}.", state.config.mister_greens_gift);
  }

  state.save();
}

