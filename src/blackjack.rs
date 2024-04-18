use crate::cards::{Card, Shoe, Value};
use crate::config::Config;
use crate::money::Money;
use crate::statistics::Statistics;
use anyhow::{bail, ensure, Context};
use anyhow::Result;
use colored::*;
use inquire::{Confirm, Select, Text};
use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::fmt;
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;


#[derive(Clone, Debug, Default)]
pub struct Hand {
    pub cards: Vec<Card>,
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

    pub fn is_bust(&self) -> bool {
        self.blackjack_sum() > 21
    }

    pub fn can_double_down(&self) -> bool {
        let player_sum = self.blackjack_sum();
        self.cards.len() == 2 && !self.doubling_down && (player_sum == 10 || player_sum == 11)
    }

    pub fn can_split(&self) -> bool {
        self.cards.len() == 2 && self.cards[0].value == self.cards[1].value
    }

    pub fn split(&mut self) -> Hand {
        let moved_card = self.cards.pop().expect("Hand needs cards to split!");

        let mut other_hand = Hand::default();
        other_hand.push(moved_card);

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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Blackjack {
    #[serde(skip)]
    config: BlackjackConfig,
    shoe: Shoe,
    #[serde(skip)]
    dealer_hand: Hand,
    #[serde(skip)]
    player_hands: Vec<Hand>,
    #[serde(skip)]
    bet: Money,
    #[serde(skip)]
    insurance: bool,
    #[serde(skip)]
    splitting: bool,
    #[serde(skip)]
    doubling_down: bool,
    #[serde(skip)]
    current_hand: usize,
}

impl Blackjack {
    pub fn new(config: BlackjackConfig) -> Self {
        Self {
            config: config.clone(),
            shoe: Shoe::new(config.shoe_count, config.shuffle_at_penetration),
            dealer_hand: Hand::new_hidden(1),
            player_hands: vec![Hand::new()],
            bet: Money::ZERO,
            insurance: false,
            splitting: false,
            doubling_down: false,
            current_hand: 0,
        }
    }

    pub fn set_shoe(&mut self, shoe: Shoe) {
        self.shoe = shoe;
    }

    pub fn set_bet(&mut self, bet: Money) {
        assert!(bet.is_sign_positive());
        assert!(!bet.is_zero());

        self.bet = bet;
    }

    pub fn current_hand_index(&self) -> usize {
        self.current_hand
    }

    pub fn current_player_hand(&self) -> &Hand {
        &self.player_hands[self.current_hand]
    }

    pub fn card_to_dealer(&mut self) {
        let card = self.shoe.draw_card();
        self.dealer_hand.push(card);
    }

    fn card_to_player(&mut self) {
        let card = self.shoe.draw_card();
        self.player_hands[self.current_hand].push(card);
    }

    pub fn hit(&mut self) {
        self.card_to_player();
    }

    pub fn initial_deal(&mut self) {
        assert!(self.dealer_hand.cards.is_empty(), "Can't do the initial deal when cards have already been dealt.");

        self.card_to_dealer();
        self.card_to_player();
        self.card_to_dealer();
        self.card_to_player();
    }

    pub fn can_place_insurance_bet(&self) -> bool {
        match self.dealer_hand.face_card().value {
            Value::Ace => true,
            _ => false,
        }
    }

    pub fn place_insurance_bet(&mut self) {
        self.insurance = true;
    }

    pub fn can_double_down(&self) -> bool {
        self.player_hands[self.current_hand].can_double_down()
    }

    pub fn can_split(&self) -> bool {
        !self.splitting
            && self.player_hands[self.current_hand].can_split()
    }

    pub fn double_down(&mut self) {
        self.card_to_player();
        self.doubling_down = true;
    }

    pub fn split(&mut self) {
        self.splitting = true;

        let mut new_hand = self.player_hands[self.current_hand].split();
        let card = self.shoe.draw_card();
        new_hand.cards.push(card);

        self.player_hands.push(new_hand);

        self.card_to_player();
    }

    pub fn stand(&mut self) {
        self.player_hands[self.current_hand].standing = true;
    }

    pub fn reveal_hole_card(&mut self) {
        self.dealer_hand.hidden_count = 0;
    }

    pub fn next_hand(&mut self) {
        self.current_hand += 1;
    }

    pub fn insurance_payout(&self) -> Money {
        self.bet + self.bet * self.config.insurance_payout_ratio
    }

    pub fn natural_blackjack_payout(&self) -> Money {
        self.bet + self.bet * self.config.blackjack_payout_ratio
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Casino {
    pub config: Config,
    pub bankroll: Money,
    blackjack: Blackjack,
    #[serde(skip)]
    pub stats: Statistics,
}

impl Casino {
    fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            bankroll: config.mister_greens_gift,
            blackjack: Blackjack::new(config.blackjack),
            ..Default::default()
        }
    }

    pub fn from_filesystem() -> Result<Self> {
        let config = Config::init_get()?;


        let mut casino =
            if config.save_path.try_exists()? {
                Self::load(&config.save_path)?
            } else {
                Self::new(config.clone())
            };

        Statistics::init(&config.stats_path)?;
        casino.stats = Statistics::load(&config.stats_path)?;

        Ok(casino)
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

    pub fn load(path: &Path) -> Result<Self> {
        let state_string = fs::read_to_string(path)?;
        let state: Self = toml::from_str(&state_string)
            .with_context(|| "Unable to parse Casino state from file")?;

        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let path = &self.config.save_path;

        fs::create_dir_all(path.parent().unwrap())
            .with_context(|| "Couldn't create save directory")?;
        fs::write(
            path,
            toml::to_string(&self).expect("Couldn't serialize save data!"),
        ).with_context(|| "Failed to write state to path")?;

        self.stats.save(&self.config.stats_path)
            .with_context(|| "Failed to save stats")?;

        Ok(())
    }

    pub fn play_blackjack(&mut self) -> Result<()> {
        println!("Your money: {}", self.bankroll);

        loop {
            let bet_text = Text::new("How much will you bet?").prompt()?;

            let bet = bet_text.trim().parse::<Money>()
                .with_context(|| "Failed to parse prompt text into an integer")?;

            if self.bankroll >= bet {
                self.blackjack.set_bet(bet);
                break;
            } else {
                println!("You can't bet that amount, try again.");
            }
        }

        println!("Betting {}", self.blackjack.bet);

        let mut sp = Spinner::new(Spinners::Dots, "Dealing cards...".into());
        sleep(Duration::from_millis(1_500));
        sp.stop_with_message(format!("{}", "* The dealer issues your cards.".dimmed()).into());

        self.blackjack.initial_deal();

        println!("Dealer's hand: {}", self.blackjack.dealer_hand);
        println!("Your hand: {}", self.blackjack.player_hands[0]);

        if self.bankroll >= self.blackjack.bet && self.blackjack.can_place_insurance_bet() {
            let take_insurance = Confirm::new("Insurance?").with_default(false).prompt()?;

            if take_insurance {
                self.bankroll -= self.blackjack.bet;
                self.blackjack.place_insurance_bet();
                println!(
                    "You make an additional {} insurance bet.",
                    self.blackjack.bet,
                );
            } else {
                println!("You choose for forego making an insurance bet.");
            }
        }

        while !self.blackjack.player_hands.iter().all(|hand| hand.is_finished()) {
            let mut options = vec!["Hit", "Stand"];

            if self.bankroll >= self.blackjack.bet && self.blackjack.can_double_down() {
                options.push("Double");
            }

            if self.bankroll >= self.blackjack.bet && self.blackjack.can_split() {
                options.push("Split");
            }

            let prompt = format!("What will you do with hand № {}?", self.blackjack.current_hand_index() + 1);

            let ans = Select::new(&prompt, options).prompt()?;

            match ans {
                "Hit" => {
                    let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
                    sleep(Duration::from_millis(1_000));
                    sp.stop_with_message(
                        format!("{}", "* The dealer hands you another card.".dimmed()).into(),
                    );

                    self.blackjack.hit();
                }
                "Double" => {
                    println!(
                        "Your bet is now {}, and you will only receive one more card.",
                        self.blackjack.bet * 2u32
                    );

                    let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
                    sleep(Duration::from_millis(1_000));
                    sp.stop_with_message(
                        format!("{}", "* The dealer hands you another card.".dimmed()).into(),
                    );
                    self.bankroll -= self.blackjack.bet;
                    self.blackjack.double_down();
                }
                "Split" => {
                    println!(
                        "You split hand № {} and place an additional {} bet.",
                        self.blackjack.current_hand_index() + 1,
                        self.blackjack.bet
                    );

                    let mut sp = Spinner::new(Spinners::Dots, "Dealing your cards...".into());
                    sleep(Duration::from_millis(1_000));
                    sp.stop_with_message(
                        format!("{}", "* The dealer hands you another two cards.".dimmed()).into(),
                    );

                    self.bankroll -= self.blackjack.bet;
                    self.blackjack.split();

                }
                "Stand" => {
                    self.blackjack.stand();
                }
                _ => bail!("Unknown answer received"),
            }

            println!(
                "Your hand № {}: {}",
                self.blackjack.current_hand_index() + 1,
                self.blackjack.current_player_hand(),
            );

            if self.blackjack.current_player_hand().is_bust() {
                let bet = self.blackjack.bet;
                self.stats.blackjack.record_loss(bet);
                self.stats.update_bankroll(self.bankroll);

                println!(
                    "HAND № {} BUST! You lose {}. You now have {}",
                    self.blackjack.current_hand_index() + 1,
                    bet,
                    self.bankroll
                );
            }

            if self.blackjack.current_player_hand().is_finished() {
                self.blackjack.next_hand();
            }
        }

        if self
            .blackjack
            .player_hands
            .iter()
            .any(|hand| !hand.is_bust())
        {
            let mut sp = Spinner::new(Spinners::Dots, "Revealing the hole card...".into());
            sleep(Duration::from_millis(1_000));
            sp.stop_with_message(format!("{}", "* Hole card revealed!".dimmed()).into());

            self.blackjack.reveal_hole_card();
            println!("Dealer's hand: {}", self.blackjack.dealer_hand);

            while self.blackjack.dealer_hand.blackjack_sum() < 17 {
                let mut sp = Spinner::new(Spinners::Dots, "Dealing another card...".into());
                sleep(Duration::from_millis(1_000));
                sp.stop_with_message(
                    format!("{}", "* The dealer issues themself another card.".dimmed()).into(),
                );

                self.blackjack.card_to_dealer();
                println!("Dealer's hand: {}", self.blackjack.dealer_hand);
            }

            let mut sp = Spinner::new(Spinners::Dots, "Determining outcome...".into());
            sleep(Duration::from_millis(1_000));
            sp.stop_with_message(format!("{}", "* The hand is finished!".dimmed()).into());

            for i in 0..self.blackjack.player_hands.iter().filter(|h| !h.is_bust()).count() {
                let hand = &self.blackjack.player_hands[i];

                if self.blackjack.dealer_hand.is_bust() {
                    let payout = self.blackjack.bet + self.blackjack.bet;
                    self.stats.blackjack.record_win(payout);
                    self.add_bankroll(payout);
                    println!(
                        "DEALER BUST! You receive {}. You now have {}",
                        payout, self.bankroll
                    );
                } else if self.blackjack.dealer_hand.blackjack_sum() == hand.blackjack_sum() {
                    self.stats.blackjack.record_push();
                    self.bankroll += self.blackjack.bet;
                    println!("PUSH! Nobody wins.");
                } else if self.blackjack.dealer_hand.blackjack_sum() > hand.blackjack_sum() {
                    let payout = self.blackjack.bet + self.blackjack.bet;
                    self.stats.blackjack.record_loss(payout);
                    self.stats.update_bankroll(self.bankroll);
                    println!(
                        "HOUSE WINS! You lose {}. You now have {}",
                        payout, self.bankroll
                    );
                } else if hand.is_natural_blackjack() {
                    let payout = self.blackjack.natural_blackjack_payout();
                    self.stats.blackjack.record_win(payout);
                    self.add_bankroll(payout);
                    println!(
                        "BLACKJACK! You receive {}. You now have {}",
                        payout, self.bankroll
                    );
                } else {
                    let payout = self.blackjack.bet + self.blackjack.bet;
                    self.stats.blackjack.record_win(payout);
                    self.add_bankroll(payout);
                    println!(
                        "YOU WIN! You receive {}. You now have {}",
                        payout, self.bankroll
                    );
                }
            }

            if self.blackjack.dealer_hand.is_natural_blackjack() && self.blackjack.insurance {
                let insurance_payout = self.blackjack.insurance_payout();
                self.bankroll += insurance_payout;
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

        self.save()?;
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

