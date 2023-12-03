use std::{fmt, fs, fs::OpenOptions, io, io::prelude::*, io::Read, io::SeekFrom, io::Write};
use rust_decimal::prelude::*;
use clap::Parser;
use directories::{ProjectDirs};
use casino::cards::{Card, Value, shoe};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

}

fn main() {
  let args = Args::parse();

  let project_dirs = ProjectDirs::from("", "", "casino").unwrap();
  let data_dir = project_dirs.data_dir();

  fs::create_dir_all(data_dir).expect("Couldn't create data dir!");

  let bankroll_path = data_dir.join("bankroll");

  let mut bankroll: i32 =
    match fs::read_to_string(&bankroll_path) {
      Ok(bankroll_string) => {
        bankroll_string.trim().parse().unwrap()
      },
      Err(_) => {
        fs::write(&bankroll_path, "1000\n").expect("Couldn't initialize your bankroll file");
        1_000
      }
    };

  println!("Your money: ${bankroll}.00");
  let mut bet: i32;

  loop {
    println!("Enter your bet: ");
    let mut bet_input = String::new();
    io::stdin().read_line(&mut bet_input).unwrap();

    bet = bet_input.trim().parse().unwrap();
    if bet <= 0 {
      println!("Try again, wiseguy")
    } else if bet > bankroll {
      println!("You don't have that much money! Try again.");
    } else {
      break;
    }
  }

  println!("Betting {bet}");

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

  let mut insurance_flag = false;

  if matches!(dealer_shown.value, Value::Ace) {
    println!("Insurance? [y/n]");
    let mut insurance_input = String::new();
    io::stdin().read_line(&mut insurance_input).unwrap();
    match insurance_input.trim() {
      "y" => {
        println!("You make an additional ${bet}.00 insurance bet.");
        insurance_flag = true;
      }
      _ => {
        println!("You choose for forgo making an insurance bet.");
        insurance_flag = false;
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
        your_hand.push(shoe.pop().unwrap());
        let sum = blackjack_sum(&your_hand);
        println!("Your hand: {} ({})", hand_to_string(&your_hand), sum);

        if sum > 21 {
          bankroll -= bet.clone();
          println!("BUST! You lose ${bet}.00. You now have ${bankroll}.00");
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
      bankroll += bet.clone();
      println!("DEALER BUST! You receive ${bet}.00. You now have ${bankroll}.00");
    } else if dealer_sum == player_sum {
      println!("PUSH! Nobody wins.");
    } else if dealer_sum > player_sum {
      bankroll -= bet.clone();
      println!("YOU LOSE! You lose ${bet}.00. You now have ${bankroll}.00");
    } else if your_hand.len() == 2 && player_sum == 21 {
      let payout = (bet.clone() * 2 / 3);
      bankroll += payout.clone();
      println!("BLACKJACK! You receive ${payout}.00. You now have ${bankroll}.00");
    } else {
      bankroll += bet.clone();
      println!("YOU WIN! You receive ${bet}. You now have ${bankroll}");
    }

    if dealer_hand.len() == 2 && dealer_sum == 21 && insurance_flag {
      let insurance_payout = bet.clone() * 2i32;
      bankroll += insurance_payout.clone();
      println!("DEALER BLACKJACK! Your insurance bet pays out ${insurance_payout}.00. You now have ${bankroll}.00.");
    }
  }


  if bankroll.is_zero() {
    bankroll += 1_000;
    println!("* Unfortunately, you've run out of money.");
    println!("* However, a portly gentleman in a sharp suit was watching you play your final hand.");
    println!("* He says \"I like your moxie, kiddo. Take this, and be a little more careful next time. This stuff doesn't grow on trees.\"");
    println!("* \"Oh, and always remember the name: MISTER GREEN!\"");
    println!("* The man hands you $1000.00.");
  }

  fs::write(bankroll_path, bankroll.to_string().as_bytes());
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
