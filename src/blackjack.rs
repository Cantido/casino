use crate::cards::{shoe, Card, Value};
use crate::config::Config;
use crate::money::Money;
use crate::statistics::Statistics;
use anyhow::ensure;
use anyhow::Result;
use colored::*;
use inquire::{Confirm, Select, Text};
use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::fmt;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
pub struct Hand {
    pub cards: Vec<Card>,
    pub bet: Money,
    pub hidden_count: usize,
    pub standing: bool,
    pub doubling_down: bool,
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

    pub fn add_bet(&mut self, amount: Money) {
        self.bet += amount;
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

        return sum;
    }

    pub fn can_double_down(&self) -> bool {
        let player_sum = self.blackjack_sum();
        self.cards.len() == 2 && !self.doubling_down && (player_sum == 10 || player_sum == 11)
    }

    pub fn double_down(&mut self) {
        self.doubling_down = true;
        self.bet *= 2;
    }

    pub fn can_split(&self) -> bool {
        self.cards.len() == 2 && self.cards[0].value == self.cards[1].value
    }

    pub fn split(&mut self) -> Hand {
        let moved_card = self.cards.pop().expect("Hand needs cards to split!");

        let mut other_hand = Hand::default();
        other_hand.push(moved_card);
        other_hand.bet = self.bet.clone();

        other_hand
    }

    pub fn is_finished(&self) -> bool {
        self.standing || self.blackjack_sum() > 21
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

        hand_str.push_str(" (");

        for (i, card) in self.cards.iter().enumerate() {
            if i < self.hidden_count {
                hand_str.push_str("?");
            } else {
                hand_str.push_str(&card.blackjack_value().to_string());
            }

            if i < self.cards.len() - 1 {
                hand_str.push_str(" + ");
            }
        }

        if self.hidden_count > 0 {
            hand_str.push_str(" = ?)");
        } else {
            hand_str.push_str(&format!(" = {})", self.blackjack_sum()));
        }

        write!(f, "{}", hand_str)
    }
}

#[derive(Default)]
pub struct Casino {
    pub config: Config,
    pub bankroll: Money,
    shoe: Vec<Card>,
    insurance_bet: Money,
    splitting: bool,
    pub stats: Statistics,
    dealer_hand: Hand,
    player_hands: Vec<Hand>,
}

impl Casino {
    fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            bankroll: config.mister_greens_gift,
            shoe: shoe(config.blackjack.shoe_count),
            dealer_hand: Hand::new_hidden(1),
            player_hands: vec![Hand::new()],
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
        Statistics::init(&self.config.stats_path).unwrap();
        self.stats = Statistics::load(&self.config.stats_path).unwrap();
    }

    fn draw_card(&mut self) -> Card {
        let card = self.shoe.pop().unwrap();

        if self.shoe.len() < self.config.blackjack.shuffle_shoe_threshold_count() {
            self.shuffle_shoe();
        }

        return card;
    }

    pub fn shuffle_shoe(&mut self) {
        self.shoe = shoe(self.config.blackjack.shoe_count);
    }

    fn card_to_dealer(&mut self) {
        let card = self.draw_card();
        self.dealer_hand.push(card);
    }

    fn card_to_player(&mut self, hand_index: usize) {
        let card = self.draw_card();
        self.player_hands[hand_index].push(card);
    }

    pub fn add_bankroll(&mut self, amount: Money) {
        self.bankroll += amount;
        self.stats.update_bankroll(self.bankroll);
    }

    pub fn subtract_bankroll(&mut self, amount: Money) -> Result<()> {
        ensure!(self.bankroll >= amount, "Cannot subtract to negative value");

        self.bankroll -= amount;
        self.stats.update_bankroll(self.bankroll);

        Ok(())
    }

    fn can_increase_bet(&self, amount: Money) -> bool {
        amount.is_sign_positive() && !amount.is_zero() && amount <= self.bankroll
    }

    fn increase_bet(&mut self, hand_index: usize, amount: Money) {
        self.player_hands[hand_index].bet += amount;
        self.bankroll -= amount;
    }

    fn can_place_insurance_bet(&self) -> bool {
        match self.dealer_hand.face_card().value {
            Value::Ace => self.player_hands[0].bet <= self.bankroll,
            _ => false,
        }
    }

    fn place_insurance_bet(&mut self) {
        let bet_amount = self.player_hands[0].bet / 2;
        self.insurance_bet += bet_amount;
        self.bankroll -= bet_amount;
    }

    fn can_double_down(&self, hand_index: usize) -> bool {
        self.player_hands[hand_index].can_double_down()
            && self.can_increase_bet(self.player_hands[hand_index].bet)
    }

    fn double_down(&mut self, hand_index: usize) {
        self.bankroll -= self.player_hands[hand_index].bet;
        self.player_hands[hand_index].double_down();
    }

    fn can_split(&self, hand_index: usize) -> bool {
        !self.splitting
            && self.can_increase_bet(self.player_hands[hand_index].bet)
            && self.player_hands[hand_index].can_split()
    }

    fn split(&mut self, hand_index: usize) {
        self.bankroll -= self.player_hands[hand_index].bet;

        let new_hand = self.player_hands[hand_index].split();
        self.player_hands.push(new_hand);
    }

    fn lose_bet(&mut self, hand_index: usize) {
        self.stats
            .blackjack
            .record_loss(self.player_hands[hand_index].bet);
        self.stats.update_bankroll(self.bankroll);
        self.player_hands[hand_index].bet = Money::ZERO;
    }

    fn win_bet(&mut self, hand_index: usize) {
        let mut payout = if self.player_hands[hand_index].is_natural_blackjack() {
            self.player_hands[hand_index].bet * self.config.blackjack.blackjack_payout_ratio
        } else {
            self.player_hands[hand_index].bet * self.config.blackjack.payout_ratio
        };

        payout += self.player_hands[hand_index].bet;

        self.stats.blackjack.record_win(payout);
        self.add_bankroll(payout);
        self.player_hands[hand_index].bet = Money::ZERO;
    }

    fn win_insurance(&mut self) {
        self.add_bankroll(self.insurance_bet + self.insurance_payout());
        self.insurance_bet = Money::ZERO;
    }

    fn push_bet(&mut self, hand_index: usize) {
        self.stats.blackjack.record_push();
        self.bankroll += self.player_hands[hand_index].bet;
        self.player_hands[hand_index].bet = Money::ZERO;
    }

    fn blackjack_payout(&self, hand_index: usize) -> Money {
        self.player_hands[hand_index].bet * self.config.blackjack.blackjack_payout_ratio
    }

    fn insurance_payout(&self) -> Money {
        self.insurance_bet * self.config.blackjack.insurance_payout_ratio
    }

    pub fn save(&self) {
        let state = CasinoState {
            bankroll: self.bankroll,
            shoe: self.shoe.clone(),
        };
        let save_dir = self
            .config
            .save_path
            .parent()
            .expect("Couldn't find save directory!");
        fs::create_dir_all(save_dir).expect("Couldn't create save directory!");
        fs::write(
            &self.config.save_path,
            toml::to_string(&state).expect("Couldn't serialize save data!"),
        )
        .expect("Couldn't write save data to save directory!");

        let stats_dir = self
            .config
            .stats_path
            .parent()
            .expect("Couldn't access stats path!");
        fs::create_dir_all(stats_dir).expect("Couldn't create stats directory!");
        fs::write(
            &self.config.stats_path,
            toml::to_string(&self.stats).unwrap(),
        )
        .expect("Couldn't write to stats file!");
    }

    pub fn play_blackjack(&mut self) -> Result<()> {
        println!("Your money: {}", self.bankroll);

        loop {
            let bet_result = Text::new("How much will you bet?").prompt();

            match bet_result {
                Ok(bet_text) => {
                    let bet = bet_text.trim().parse::<Money>().unwrap();
                    if self.can_increase_bet(bet) {
                        self.increase_bet(0, bet);
                        break;
                    } else {
                        println!("You can't bet that amount, try again.");
                    }
                }
                Err(_) => panic!("Error getting your answer."),
            }
        }

        println!("Betting {}", self.player_hands[0].bet);

        let mut sp = Spinner::new(Spinners::Dots, "Dealing cards...".into());
        sleep(Duration::from_millis(1_500));
        sp.stop_with_message(format!("{}", "* The dealer issues your cards.".dimmed()).into());

        self.card_to_dealer();
        self.card_to_player(0);
        self.card_to_dealer();
        self.card_to_player(0);

        println!("Dealer's hand: {}", self.dealer_hand);
        println!("Your hand: {}", self.player_hands[0]);

        if self.can_place_insurance_bet() {
            let ans = Confirm::new("Insurance?").with_default(false).prompt();

            match ans {
                Ok(true) => {
                    self.place_insurance_bet();
                    println!(
                        "You make an additional {} insurance bet.",
                        self.insurance_bet
                    );
                }
                Ok(false) => println!("You choose for forego making an insurance bet."),
                Err(_) => panic!("Error getting your answer"),
            }
        }

        let mut current_hand = 0;

        while !self.player_hands.iter().all(|hand| hand.is_finished()) {
            let mut options = vec!["Hit", "Stand"];

            if self.can_double_down(current_hand) {
                options.push("Double");
            }

            if self.can_split(current_hand) {
                options.push("Split");
            }

            let prompt = format!("What will you do with hand № {}?", current_hand + 1);

            let ans = Select::new(&prompt, options).prompt();

            match ans {
                Ok("Hit") => {
                    let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
                    sleep(Duration::from_millis(1_000));
                    sp.stop_with_message(
                        format!("{}", "* The dealer hands you another card.".dimmed()).into(),
                    );

                    self.card_to_player(current_hand);
                }
                Ok("Double") => {
                    self.double_down(current_hand);
                    println!(
                        "Your bet is now {}, and you will only receive one more card.",
                        self.player_hands[0].bet
                    );

                    let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
                    sleep(Duration::from_millis(1_000));
                    sp.stop_with_message(
                        format!("{}", "* The dealer hands you another card.".dimmed()).into(),
                    );

                    self.card_to_player(current_hand);

                    self.player_hands[current_hand].standing = true;
                }
                Ok("Split") => {
                    self.split(current_hand);
                    println!(
                        "You split hand № {} and place an additional {} bet.",
                        current_hand + 1,
                        self.player_hands[1].bet
                    );

                    let mut sp = Spinner::new(Spinners::Dots, "Dealing your cards...".into());
                    sleep(Duration::from_millis(1_000));
                    sp.stop_with_message(
                        format!("{}", "* The dealer hands you another two cards.".dimmed()).into(),
                    );

                    self.card_to_player(current_hand);
                    self.card_to_player(current_hand + 1);
                }
                Ok("Stand") => {
                    self.player_hands[current_hand].standing = true;
                }
                Ok(_) => panic!("Unknown answer received"),
                Err(_) => panic!("Error getting your answer."),
            }

            println!(
                "Your hand № {}: {}",
                current_hand + 1,
                self.player_hands[current_hand]
            );

            if self.player_hands[current_hand].blackjack_sum() > 21 {
                let bet = self.player_hands[current_hand].bet;
                self.lose_bet(current_hand);
                println!(
                    "HAND № {} BUST! You lose {}. You now have {}",
                    current_hand + 1,
                    bet,
                    self.bankroll
                );
            }

            if self.player_hands[current_hand].is_finished() {
                current_hand += 1;
            }
        }

        if self
            .player_hands
            .iter()
            .any(|hand| hand.blackjack_sum() <= 21)
        {
            let mut sp = Spinner::new(Spinners::Dots, "Revealing the hole card...".into());
            sleep(Duration::from_millis(1_000));
            sp.stop_with_message(format!("{}", "* Hole card revealed!".dimmed()).into());

            self.dealer_hand.hidden_count = 0;
            println!("Dealer's hand: {}", self.dealer_hand);

            while self.dealer_hand.blackjack_sum() < 17 {
                let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
                sleep(Duration::from_millis(1_000));
                sp.stop_with_message(
                    format!("{}", "* The dealer issues themself another card.".dimmed()).into(),
                );

                self.card_to_dealer();
                println!("Dealer's hand: {}", self.dealer_hand);
            }

            let mut sp = Spinner::new(Spinners::Dots, "Determining outcome...".into());
            sleep(Duration::from_millis(1_000));
            sp.stop_with_message(format!("{}", "* The hand is finished!".dimmed()).into());

            for i in 0..self.player_hands.len() {
                let hand = &self.player_hands[i];

                if hand.blackjack_sum() <= 21 {
                    if self.dealer_hand.blackjack_sum() > 21 {
                        let bet = hand.bet;
                        self.win_bet(i);
                        println!(
                            "DEALER BUST! You receive {}. You now have {}",
                            bet, self.bankroll
                        );
                    } else if self.dealer_hand.blackjack_sum() == hand.blackjack_sum() {
                        self.push_bet(i);
                        println!("PUSH! Nobody wins.");
                    } else if self.dealer_hand.blackjack_sum() > hand.blackjack_sum() {
                        let bet = hand.bet;
                        self.lose_bet(i);
                        println!(
                            "HOUSE WINS! You lose {}. You now have {}",
                            bet, self.bankroll
                        );
                    } else if hand.is_natural_blackjack() {
                        let payout = self.blackjack_payout(i);
                        self.win_bet(i);
                        println!(
                            "BLACKJACK! You receive {}. You now have {}",
                            payout, self.bankroll
                        );
                    } else {
                        let bet = hand.bet;
                        self.win_bet(i);
                        println!(
                            "YOU WIN! You receive {}. You now have {}",
                            bet, self.bankroll
                        );
                    }
                }
            }

            if self.dealer_hand.is_natural_blackjack() && !self.insurance_bet.is_zero() {
                let insurance_payout = self.insurance_payout();
                self.win_insurance();
                println!(
                    "DEALER BLACKJACK! Your insurance bet pays out {}. You now have {}.",
                    insurance_payout, self.bankroll
                );
            }
        }

        if self.bankroll.is_zero() {
            self.add_bankroll(self.config.mister_greens_gift);
            println!("{}", "* Unfortunately, you've run out of money.".dimmed());
            println!("{}", "* However, a portly gentleman in a sharp suit was watching you play your final hand.".dimmed());
            println!("{}", "* He says \"I like your moxie, kiddo. Take this, and be a little more careful next time. This stuff doesn't grow on trees.\"".dimmed());
            println!(
                "{}",
                "* \"Oh, and always remember the name: MISTER GREEN!\"".dimmed()
            );
            println!(
                "{}",
                format!("* The man hands you {}", self.config.mister_greens_gift).dimmed()
            );
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
