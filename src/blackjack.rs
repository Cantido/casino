use std::fmt;
use std::fs;
use anyhow::Result;
use inquire::{Confirm, Select, Text};
use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::thread::sleep;
use std::time::Duration;
use crate::money::Money;
use crate::cards::{Card, Value, shoe};
use crate::statistics::BlackjackStatistics;
use crate::config::Config;

#[derive(Default)]
pub struct Hand {
  pub cards: Vec<Card>,
  pub hidden_count: usize,
}

impl Hand {
  pub fn new() -> Self {
    Hand::default()
  }

  pub fn new_hidden(hidden_count: usize) -> Self {
    let mut hand = Hand::default();
    hand.hidden_count = hidden_count;
    hand
  }

  pub fn push(&mut self, card: Card) {
    self.cards.push(card);
  }

  pub fn face_card(&self) -> &Card {
    &self.cards[1]
  }

  pub fn is_natural_blackjack(&self) -> bool {
    self.cards.len() == 2 && self.blackjack_sum() == 21
  }

  pub fn blackjack_sum(&self) -> u8 {
    let mut sum = 0;
    for card in self.cards.iter() {
      sum += card.blackjack_value();
    }

    let has_ace = self.cards.iter().any(|c| matches!(&c.value, Value::Ace));

    if has_ace && sum <= 11 {
      sum += 10;
    }

    return sum
  }
}

impl fmt::Display for Hand {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut hand_str: String = "".to_owned();

    for (i, card) in self.cards.iter().enumerate() {
      if i < self.hidden_count {
        hand_str.push_str("🂠 ");
      } else {
        hand_str.push_str(&card.to_string());
      }
    }

    write!(f, "{}", hand_str)
  }
}

#[derive(Default)]
pub struct Casino {
  config: Config,
  pub bankroll: Money,
  shoe: Vec<Card>,
  bet: Money,
  insurance_bet: Money,
  split_bet: Money,
  standing: bool,
  standing_split: bool,
  doubling_down: bool,
  splitting: bool,
  pub stats: BlackjackStatistics,
  dealer_hand: Hand,
  player_hand: Hand,
  split_hand: Hand,
}

impl Casino {
  fn new(config: Config) -> Self {
    Self {
      config: config.clone(),
      bankroll: config.mister_greens_gift,
      shoe: shoe(config.blackjack.shoe_count),
      dealer_hand: Hand::new_hidden(1),
      ..Default::default()
    }
  }

  pub fn from_filesystem() -> Result<Self> {
    let config = Config::init_get().expect("Couldn't init config file");
    let mut casino = Self::new(config);

    casino.load_state();
    casino.load_stats();

    Ok(casino)
  }

  fn load_state(&mut self) {
    if let Ok(state_string) = fs::read_to_string(&self.config.save_path) {
      let state: CasinoState = toml::from_str(&state_string).unwrap();

      self.bankroll = state.bankroll;
      self.shoe = state.shoe.clone();
    } else {
      println!("Couldn't read save file!");
    }
  }

  fn load_stats(&mut self) {
    if let Ok(stats_string) = fs::read_to_string(&self.config.stats_path) {
      let stats: BlackjackStatistics = toml::from_str(&stats_string).unwrap();

      self.stats = stats;
    };
  }

  fn draw_card(&mut self) -> Card {
    let card = self.shoe.pop().unwrap();

    if self.shoe.len() < self.config.blackjack.shuffle_shoe_threshold_count() {
      self.shuffle_shoe();
    }

    return card
  }

  pub fn shuffle_shoe(&mut self) {
    self.shoe = shoe(self.config.blackjack.shoe_count);
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

  fn add_bankroll(&mut self, amount: Money) {
    self.bankroll += amount;
    self.stats.update_bankroll(self.bankroll);
  }

  fn can_increase_bet(&self, amount: Money) -> bool {
    amount.is_sign_positive() && !amount.is_zero() && amount <= self.bankroll
  }

  fn increase_bet(&mut self, amount: Money) {
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
    let bet_amount = self.bet / 2;
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
    self.bet *= 2;
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
    self.bet = Money::ZERO;
  }

  fn lose_split_bet(&mut self) {
    self.stats.record_loss(self.split_bet);
    self.stats.update_bankroll(self.bankroll);
    self.split_bet = Money::ZERO;
  }

  fn win_bet(&mut self) {
    self.stats.record_win(self.win_payout());
    self.bankroll += self.bet + self.win_payout();
    self.stats.update_bankroll(self.bankroll);
    self.bet = Money::ZERO;
  }

  fn win_split_bet(&mut self) {
    self.stats.record_win(self.win_split_payout());
    self.bankroll += self.split_bet + self.win_split_payout();
    self.stats.update_bankroll(self.bankroll);
    self.split_bet = Money::ZERO;
  }

  fn win_bet_blackjack(&mut self) {
    self.stats.record_win(self.blackjack_payout());
    self.bankroll += self.bet + self.blackjack_payout();
    self.stats.update_bankroll(self.bankroll);
    self.bet = Money::ZERO;
  }
  fn win_split_bet_blackjack(&mut self) {
    self.stats.record_win(self.split_blackjack_payout());
    self.bankroll += self.split_bet + self.split_blackjack_payout();
    self.stats.update_bankroll(self.bankroll);
    self.split_bet = Money::ZERO;
  }

  fn win_insurance(&mut self) {
    self.bankroll += self.bet + self.insurance_payout();
    self.stats.update_bankroll(self.bankroll);
    self.insurance_bet = Money::ZERO;
  }

  fn push_bet(&mut self) {
    self.stats.record_push();
    self.bankroll += self.bet;
    self.bet = Money::ZERO;
  }

  fn push_split_bet(&mut self) {
    self.stats.record_push();
    self.bankroll += self.split_bet;
    self.split_bet = Money::ZERO;
  }

  fn win_payout(&self) -> Money {
    self.config.blackjack.payout(self.bet)
  }

  fn win_split_payout(&self) -> Money {
    self.config.blackjack.payout(self.split_bet)
  }

  fn blackjack_payout(&self) -> Money {
    self.config.blackjack.blackjack_payout(self.bet)
  }

  fn split_blackjack_payout(&self) -> Money {
    self.config.blackjack.blackjack_payout(self.split_bet)
  }

  fn insurance_payout(&self) -> Money {
    self.config.blackjack.insurance_payout(self.split_bet)
  }

  pub fn save(&self) {
    let state = CasinoState { bankroll: self.bankroll, shoe: self.shoe.clone() };
    let save_dir = self.config.save_path.parent().expect("Couldn't find save directory!");
    fs::create_dir_all(save_dir).expect("Couldn't create save directory!");
    fs::write(&self.config.save_path, toml::to_string(&state).expect("Couldn't serialize save data!")).expect("Couldn't write save data to save directory!");

    let stats_dir = self.config.stats_path.parent().expect("Couldn't access stats path!");
    fs::create_dir_all(stats_dir).expect("Couldn't create stats directory!");
    fs::write(&self.config.stats_path, toml::to_string(&self.stats).unwrap()).expect("Couldn't write to stats file!");
  }

  pub fn play_blackjack(&mut self) -> Result<()> {
    println!("Your money: {}", self.bankroll);

    loop {
      let bet_result = Text::new("How much will you bet?").prompt();

      match bet_result {
        Ok(bet_text) => {
          let bet = bet_text.trim().parse::<Money>().unwrap();
          if self.can_increase_bet(bet) {
            self.increase_bet(bet);
            break;
          } else {
            println!("You can't bet that amount, try again.");
          }
        },
        Err(_) => panic!("Error getting your answer."),
      }
    }

    println!("Betting {}", self.bet);

    let mut sp = Spinner::new(Spinners::Dots, "Dealing cards...".into());
    sleep(Duration::from_millis(1_500));
    sp.stop_with_message("* The dealer issues your cards.".into());

    self.card_to_dealer();
    self.card_to_player();
    self.card_to_dealer();
    self.card_to_player();

    println!("Dealer's hand: {}", self.dealer_hand);
    println!("Your hand: {} ({})", self.player_hand, self.player_hand.blackjack_sum());

    if self.can_place_insurance_bet() {
      let ans = Confirm::new("Insurance?").with_default(false).prompt();

      match ans {
        Ok(true) => {
          self.place_insurance_bet();
          println!("You make an additional {} insurance bet.", self.insurance_bet);
        },
        Ok(false) => println!("You choose for forgo making an insurance bet."),
        Err(_) => panic!("Error getting your answer"),
      }
    }

    let mut current_hand = 0;

    while !(self.standing || self.player_hand.blackjack_sum() > 21) || (self.splitting && !(self.standing_split || self.split_hand.blackjack_sum() > 21)) {

      let mut options = vec!["Hit", "Stand"];

      if self.can_double_down() {
        options.push("Double");
      }

      if self.can_split() {
        options.push("Split");
      }

      let prompt =
        if self.splitting && current_hand == 0 {
          "What will you do with your first hand?"
        } else if self.splitting && current_hand == 1 {
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

          if self.splitting && current_hand == 0 {
            self.card_to_player();
            println!("Your first hand: {} ({})", self.player_hand, self.player_hand.blackjack_sum());

            if self.player_hand.blackjack_sum() > 21 {
              let bet = self.bet;
              self.lose_bet();
              current_hand = 1;
              println!("FIRST HAND BUST! You lose {}. You now have {}", bet, self.bankroll);
            }
          } else if self.splitting && current_hand == 1 {
            self.card_to_split();
            println!("Your second hand: {} ({})", self.split_hand, self.split_hand.blackjack_sum());

            if self.split_hand.blackjack_sum() > 21 {
              let bet = self.split_bet;
              self.lose_split_bet();
              println!("SECOND HAND BUST! You lose {}. You now have {}", bet, self.bankroll);
            }
          } else {
            self.card_to_player();
            println!("Your hand: {} ({})", self.player_hand, self.player_hand.blackjack_sum());

            if self.player_hand.blackjack_sum() > 21 {
              let bet = self.bet;
              self.lose_bet();
              println!("BUST! You lose {}. You now have {}", bet, self.bankroll);
            }
          }
        },
        Ok("Double") => {
          self.double_down();
          println!("Your bet is now {}, and you will only receive one more card.", self.bet);

          let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
          sleep(Duration::from_millis(1_000));
          sp.stop_with_message("* The dealer hands you another card.".into());

          self.card_to_player();
          println!("Your hand: {} ({})", self.player_hand, self.player_hand.blackjack_sum());

          if self.player_hand.blackjack_sum() > 21 {
            let bet = self.bet;
            self.lose_bet();
            println!("BUST! You lose {}. You now have {}", bet, self.bankroll);
          }
          self.standing = true;
        },
        Ok("Split") => {
          self.split();
          println!("You split your hand and place a second {} bet.", self.split_bet);

          let mut sp = Spinner::new(Spinners::Dots, "Dealing your cards...".into());
          sleep(Duration::from_millis(1_000));
          sp.stop_with_message("* The dealer hands you another two cards.".into());

          self.card_to_player();
          self.card_to_split();

          println!("Your first hand: {} ({})", self.player_hand, self.player_hand.blackjack_sum());
          println!("Your second hand: {} ({})", self.split_hand, self.split_hand.blackjack_sum());

          if self.player_hand.blackjack_sum() > 21 {
            let bet = self.bet;
            self.lose_bet();
            println!("FIRST HAND BUST! You lose {}. You now have {}", bet, self.bankroll);
          }

          if self.split_hand.blackjack_sum() > 21 {
            let bet = self.split_bet;
            self.lose_split_bet();
            println!("SECOND HAND BUST! You lose {}. You now have {}", bet, self.bankroll);
          }
        },
        Ok("Stand") => {
          if current_hand == 0 {
            self.standing = true;
            current_hand = 1;
          } else if current_hand == 1 {
            self.standing_split = true;
          }
        },
        Ok(_) => panic!("Unknown answer received"),
        Err(_) => panic!("Error getting your answer."),
      }
    }

    if self.player_hand.blackjack_sum() <= 21 || (self.splitting && self.split_hand.blackjack_sum() <= 21) {
      let mut sp = Spinner::new(Spinners::Dots, "Revealing the hole card...".into());
      sleep(Duration::from_millis(1_000));
      sp.stop_with_message("* Hole card revealed!".into());

      self.dealer_hand.hidden_count = 0;
      println!("Dealer's hand: {} ({})", self.dealer_hand, self.dealer_hand.blackjack_sum());

      while self.dealer_hand.blackjack_sum() < 17 {
        let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
        sleep(Duration::from_millis(1_000));
        sp.stop_with_message("* The dealer issues themself another card.".into());

        self.card_to_dealer();
        println!("Dealer's hand: {} ({})", self.dealer_hand, self.dealer_hand.blackjack_sum());
      }

      let mut sp = Spinner::new(Spinners::Dots, "Determining outcome...".into());
      sleep(Duration::from_millis(1_000));
      sp.stop_with_message("* The hand is finished!".into());

      if self.player_hand.blackjack_sum() <= 21 {
        if self.splitting {
          print!("First hand result: ");
        }

        if self.dealer_hand.blackjack_sum() > 21 {
          let bet = self.bet;
          self.win_bet();
          println!("DEALER BUST! You receive {}. You now have {}", bet, self.bankroll);
        } else if self.dealer_hand.blackjack_sum() == self.player_hand.blackjack_sum() {
          self.push_bet();
          println!("PUSH! Nobody wins.");
        } else if self.dealer_hand.blackjack_sum() > self.player_hand.blackjack_sum() {
          let bet = self.bet;
          self.lose_bet();
          println!("HOUSE WINS! You lose {}. You now have {}", bet, self.bankroll);
        } else if self.player_hand.is_natural_blackjack() {
          let payout = self.blackjack_payout();
          self.win_bet_blackjack();
          println!("BLACKJACK! You receive {}. You now have {}", payout, self.bankroll);
        } else {
          let bet = self.bet;
          self.win_bet();
          println!("YOU WIN! You receive {}. You now have {}", bet, self.bankroll);
        }
      }

      if self.splitting && self.split_hand.blackjack_sum() <= 21 {
        print!("Second hand result: ");

        if self.dealer_hand.blackjack_sum() > 21 {
          let bet = self.split_bet;
          self.win_split_bet();
          println!("DEALER BUST! You receive {}. You now have {}", bet, self.bankroll);
        } else if self.dealer_hand.blackjack_sum() == self.split_hand.blackjack_sum() {
          self.push_split_bet();
          println!("PUSH! Nobody wins.");
        } else if self.dealer_hand.blackjack_sum() > self.split_hand.blackjack_sum() {
          let bet = self.split_bet;
          self.lose_split_bet();
          println!("HOUSE WINS! You lose {}. You now have {}", bet, self.bankroll);
        } else if self.split_hand.is_natural_blackjack() {
          let payout = self.split_blackjack_payout();
          self.win_split_bet_blackjack();
          println!("BLACKJACK! You receive {}. You now have {}", payout, self.bankroll);
        } else {
          let bet = self.split_bet;
          self.win_split_bet();
          println!("YOU WIN! You receive {}. You now have {}", bet, self.bankroll);
        }
      }

      if self.dealer_hand.is_natural_blackjack() && !self.insurance_bet.is_zero() {
        let insurance_payout = self.insurance_payout();
        self.win_insurance();
        println!("DEALER BLACKJACK! Your insurance bet pays out {}. You now have {}.", insurance_payout, self.bankroll);
      }
    }

    if self.bankroll.is_zero() {
      self.add_bankroll(self.config.mister_greens_gift);
      println!("* Unfortunately, you've run out of money.");
      println!("* However, a portly gentleman in a sharp suit was watching you play your final hand.");
      println!("* He says \"I like your moxie, kiddo. Take this, and be a little more careful next time. This stuff doesn't grow on trees.\"");
      println!("* \"Oh, and always remember the name: MISTER GREEN!\"");
      println!("* The man hands you {}.", self.config.mister_greens_gift);
    }

    self.save();
    Ok(())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlackjackConfig {
  #[serde(default = "BlackjackConfig::default_shoe_count")]
  pub shoe_count: u8,

  #[serde(default = "BlackjackConfig::default_shuffle_penetration")]
  pub shuffle_at_penetration: f32,

  #[serde(default = "BlackjackConfig::default_payout_ratio")]
  pub payout_ratio: Ratio<i64>,

  #[serde(default = "BlackjackConfig::default_blackjack_payout_ratio")]
  pub blackjack_payout_ratio: Ratio<i64>,

  #[serde(default = "BlackjackConfig::default_insurance_payout_ratio")]
  pub insurance_payout_ratio: Ratio<i64>,
}

impl Default for BlackjackConfig {
  fn default() -> Self {
    Self {
      shoe_count: Self::default_shoe_count(),
      shuffle_at_penetration: Self::default_shuffle_penetration(),
      payout_ratio: Self::default_payout_ratio(),
      blackjack_payout_ratio: Self::default_blackjack_payout_ratio(),
      insurance_payout_ratio: Self::default_insurance_payout_ratio(),
    }
  }
}

impl BlackjackConfig {
  pub fn shuffle_shoe_threshold_count(&self) -> usize {
    let threshold_fraction = 1f32 - self.shuffle_at_penetration;
    let starting_shoe_size = self.shoe_count as usize * 52;

    (starting_shoe_size as f32 * threshold_fraction) as usize
  }

  fn payout(&self, bet: Money) -> Money {
    bet * self.payout_ratio
  }

  fn blackjack_payout(&self, bet: Money) -> Money {
    bet * self.blackjack_payout_ratio
  }

  fn insurance_payout(&self, bet: Money) -> Money {
    bet * self.insurance_payout_ratio
  }

  fn default_shoe_count() -> u8 {
    4
  }

  fn default_shuffle_penetration() -> f32 {
    0.75
  }

  fn default_payout_ratio() -> Ratio<i64> {
    Ratio::new(1, 1)
  }

  fn default_blackjack_payout_ratio() -> Ratio<i64> {
    Ratio::new(3, 2)
  }

  fn default_insurance_payout_ratio() -> Ratio<i64> {
    Ratio::new(2, 1)
  }
}

#[derive(Deserialize, Debug, Serialize)]
struct CasinoState {
  bankroll: Money,
  shoe: Vec<Card>,
}


