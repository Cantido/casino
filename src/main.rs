use std::io;
use std::fmt;
use clap::Parser;
use casino::cards::{Card, Value, shoe};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

}

fn main() {
  let args = Args::parse();

  println!("Enter your bet: ");
  let mut bet_input = String::new();
  io::stdin().read_line(&mut bet_input).unwrap();

  let bet: i32 = bet_input.trim().parse().unwrap();

  println!("Betting ${bet}.00");

  let mut shoe = shoe(4);

  let mut dealer_hidden;
  let mut dealer_shown;
  let mut your_hand = vec![];

  dealer_hidden = shoe.pop().unwrap();
  your_hand.push(shoe.pop().unwrap());
  dealer_shown = shoe.pop().unwrap();
  your_hand.push(shoe.pop().unwrap());

  println!("Dealer's hand: 🂠 {}", dealer_shown);
  println!("Your hand: {} ({})", hand_to_string(&your_hand), blackjack_sum(&your_hand));

  loop {
    println!("Hit or stand? [h/s]:");

    let mut hit_stand_input = String::new();
    io::stdin().read_line(&mut hit_stand_input).unwrap();
    match hit_stand_input.trim() {
      "h" | "hit" => {
        println!("* The dealer deals you another card");
        your_hand.push(shoe.pop().unwrap());
        let sum = blackjack_sum(&your_hand);
        println!("Your hand: {} ({})", hand_to_string(&your_hand), sum);

        if sum > 21 {
          println!("BUST! You lose ${bet}.00");
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
      dealer_hand.push(shoe.pop().unwrap());
      dealer_sum = blackjack_sum(&dealer_hand);
      println!("Dealer's hand: {} ({})", hand_to_string(&dealer_hand), dealer_sum);
    }

    if dealer_sum > 21 {
      println!("DEALER BUST! You win ${bet}.00");
    } else if dealer_sum == player_sum {
      println!("PUSH! Nobody wins.");
    } else if dealer_sum > player_sum {
      println!("YOU LOSE! You lose ${bet}.00");
    }
  }
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
