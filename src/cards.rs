use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub fn symbol(&self) -> char {
        match self {
            Self::Clubs => '♣',
            Self::Diamonds => '♦',
            Self::Hearts => '♥',
            Self::Spades => '♠',
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Value {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ace => f.pad("A"),
            Self::Two => f.pad("2"),
            Self::Three => f.pad("3"),
            Self::Four => f.pad("4"),
            Self::Five => f.pad("5"),
            Self::Six => f.pad("6"),
            Self::Seven => f.pad("7"),
            Self::Eight => f.pad("8"),
            Self::Nine => f.pad("9"),
            Self::Ten => f.pad("10"),
            Self::Jack => f.pad("J"),
            Self::Queen => f.pad("Q"),
            Self::King => f.pad("K"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Card {
    pub suit: Suit,
    pub value: Value,
}

impl Card {
    pub fn blackjack_value(&self) -> u8 {
        return match &self.value {
            Value::Ace => 1,
            Value::Two => 2,
            Value::Three => 3,
            Value::Four => 4,
            Value::Five => 5,
            Value::Six => 6,
            Value::Seven => 7,
            Value::Eight => 8,
            Value::Nine => 9,
            Value::Ten | Value::Jack | Value::Queen | Value::King => 10,
        };
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol = match (&self.value, &self.suit) {
            (Value::Ace, Suit::Spades) => "🂡",
            (Value::Two, Suit::Spades) => "🂢",
            (Value::Three, Suit::Spades) => "🂣",
            (Value::Four, Suit::Spades) => "🂤",
            (Value::Five, Suit::Spades) => "🂥",
            (Value::Six, Suit::Spades) => "🂦",
            (Value::Seven, Suit::Spades) => "🂧",
            (Value::Eight, Suit::Spades) => "🂨",
            (Value::Nine, Suit::Spades) => "🂩",
            (Value::Ten, Suit::Spades) => "🂪",
            (Value::Jack, Suit::Spades) => "🂫",
            (Value::Queen, Suit::Spades) => "🂭",
            (Value::King, Suit::Spades) => "🂮",
            (Value::Ace, Suit::Hearts) => "🂱",
            (Value::Two, Suit::Hearts) => "🂲",
            (Value::Three, Suit::Hearts) => "🂳",
            (Value::Four, Suit::Hearts) => "🂴",
            (Value::Five, Suit::Hearts) => "🂵",
            (Value::Six, Suit::Hearts) => "🂶",
            (Value::Seven, Suit::Hearts) => "🂷",
            (Value::Eight, Suit::Hearts) => "🂸",
            (Value::Nine, Suit::Hearts) => "🂹",
            (Value::Ten, Suit::Hearts) => "🂺",
            (Value::Jack, Suit::Hearts) => "🂻",
            (Value::Queen, Suit::Hearts) => "🂽",
            (Value::King, Suit::Hearts) => "🂾",
            (Value::Ace, Suit::Diamonds) => "🃁",
            (Value::Two, Suit::Diamonds) => "🃂",
            (Value::Three, Suit::Diamonds) => "🃃",
            (Value::Four, Suit::Diamonds) => "🃄",
            (Value::Five, Suit::Diamonds) => "🃅",
            (Value::Six, Suit::Diamonds) => "🃆",
            (Value::Seven, Suit::Diamonds) => "🃇",
            (Value::Eight, Suit::Diamonds) => "🃈",
            (Value::Nine, Suit::Diamonds) => "🃉",
            (Value::Ten, Suit::Diamonds) => "🃊",
            (Value::Jack, Suit::Diamonds) => "🃋",
            (Value::Queen, Suit::Diamonds) => "🃍",
            (Value::King, Suit::Diamonds) => "🃎",
            (Value::Ace, Suit::Clubs) => "🃑",
            (Value::Two, Suit::Clubs) => "🃒",
            (Value::Three, Suit::Clubs) => "🃓",
            (Value::Four, Suit::Clubs) => "🃔",
            (Value::Five, Suit::Clubs) => "🃕",
            (Value::Six, Suit::Clubs) => "🃖",
            (Value::Seven, Suit::Clubs) => "🃗",
            (Value::Eight, Suit::Clubs) => "🃘",
            (Value::Nine, Suit::Clubs) => "🃙",
            (Value::Ten, Suit::Clubs) => "🃚",
            (Value::Jack, Suit::Clubs) => "🃛",
            (Value::Queen, Suit::Clubs) => "🃝",
            (Value::King, Suit::Clubs) => "🃞",
        };

        write!(f, "{} ", symbol)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Shoe {
    deck_count: u8,
    reshuffle_threshold: f32,
    cards: Vec<Card>,
}

impl Shoe {
    pub fn new(deck_count: u8, reshuffle_threshold: f32) -> Self {
        Self {
            deck_count,
            reshuffle_threshold,
            cards: Self::build_cards(deck_count),
        }
    }


    pub fn draw_card(&mut self) -> Card {
        if self.cards.len() < ((self.deck_count * 52) as f32 * self.reshuffle_threshold) as usize {
            self.cards = Self::build_cards(self.deck_count);
        }

        return self.cards.pop().unwrap();
    }

    pub fn shuffle(&mut self) {
        self.cards = Self::build_cards(self.deck_count);
    }

    fn build_cards(deck_count: u8) -> Vec<Card> {
        let deck = deck();

        let mut shoe = vec![];

        for _i in 1..deck_count {
            let mut next_deck = deck.clone();
            shoe.append(&mut next_deck);
        }

        let mut rng = thread_rng();
        shoe.shuffle(&mut rng);

        shoe
    }
}

pub fn deck() -> Vec<Card> {
    let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
    let values = [
        Value::Ace,
        Value::Two,
        Value::Three,
        Value::Four,
        Value::Five,
        Value::Six,
        Value::Seven,
        Value::Eight,
        Value::Nine,
        Value::Ten,
        Value::Jack,
        Value::Queen,
        Value::King,
    ];

    let mut cards = vec![];

    for suit in suits {
        for value in &values {
            let card = Card {
                suit: suit.clone(),
                value: value.clone(),
            };

            cards.push(card);
        }
    }

    let mut rng = thread_rng();
    cards.shuffle(&mut rng);
    return cards;
}

pub fn random_card() -> Card {
    let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
    let values = [
        Value::Ace,
        Value::Two,
        Value::Three,
        Value::Four,
        Value::Five,
        Value::Six,
        Value::Seven,
        Value::Eight,
        Value::Nine,
        Value::Ten,
        Value::Jack,
        Value::Queen,
        Value::King,
    ];

    let mut rng = thread_rng();
    let suit = suits.choose(&mut rng).unwrap().clone();
    let value = values.choose(&mut rng).unwrap().clone();

    return Card {
        suit,
        value,
    };
}
