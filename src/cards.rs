use std::fmt;
use rand::thread_rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Suit {
  Clubs,
  Diamonds,
  Hearts,
  Spades,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
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
      Value::Ten | Value::Jack | Value::Queen | Value::King => 10
    }
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

pub fn deck() -> Vec<Card> {
  let suits = [
    Suit::Clubs,
    Suit::Diamonds,
    Suit::Hearts,
    Suit::Spades,
  ];
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
      let card = Card { suit: suit.clone(), value: value.clone() };

      cards.push(card);
    }
  }

  let mut rng = thread_rng();
  cards.shuffle(&mut rng);
  return cards
}

pub fn shoe(deck_count: u8) -> Vec<Card> {
  let deck = deck();

  let mut shoe = vec![];

  for _i in 1..deck_count {
    let mut next_deck = deck.clone();
    shoe.append(&mut next_deck);
  }

  let mut rng = thread_rng();
  shoe.shuffle(&mut rng);
  return shoe
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
    suit: suit,
    value: value,
  }

}
