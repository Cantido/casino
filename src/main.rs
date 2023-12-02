use std::{fmt, fs, fs::OpenOptions, io, io::prelude::*, io::Read, io::SeekFrom, io::Write};
use rusty_money::{Money, Round, iso};
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

  let mut bankroll_file = OpenOptions::new().read(true).write(true).create(true).open(data_dir.join("bankroll")).expect("Couldn't open your bankroll file!");

  let mut bankroll_buffer = String::new();

  bankroll_file.read_to_string(&mut bankroll_buffer).expect("Couldn't read your bankroll file!");

  let mut bankroll =
    if bankroll_buffer.is_empty() {
      bankroll_file.write_all(b"$1000.00\n");
      Money::from_major(1_000, iso::USD)
    } else {
      let stramt = bankroll_buffer.trim().trim_start_matches('$');
      Money::from_str(stramt, iso::USD).unwrap()
    };

  println!("Your money: {bankroll}");
  let mut bet;

  loop {
    println!("Enter your bet: ");
    let mut bet_input = String::new();
    io::stdin().read_line(&mut bet_input).unwrap();

    bet = Money::from_str(bet_input.trim().trim_start_matches('$'), iso::USD).unwrap().round(2, Round::HalfUp);
    if bet.is_negative() || bet.is_zero() {
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
        println!("You make an additional {bet} insurance bet.");
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
          println!("BUST! You lose {bet}. You now have {bankroll}");
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
      println!("DEALER BUST! You receive {bet}. You now have {bankroll}");
    } else if dealer_sum == player_sum {
      println!("PUSH! Nobody wins.");
    } else if dealer_sum > player_sum {
      bankroll -= bet.clone();
      println!("YOU LOSE! You lose {bet}. You now have {bankroll}");
    } else if your_hand.len() == 2 && player_sum == 21 {
      let payout_ratio = Decimal::new(15, 1);
      let payout = (bet.clone() * payout_ratio).round(2, Round::HalfUp);
      bankroll += payout.clone();
      println!("BLACKJACK! You receive {payout}. You now have {bankroll}");
    } else {
      bankroll += bet.clone();
      println!("YOU WIN! You receive {bet}. You now have {bankroll}");
    }

    if dealer_hand.len() == 2 && dealer_sum == 21 && insurance_flag {
      let insurance_payout = bet.clone() * 2i32;
      bankroll += insurance_payout.clone();
      println!("DEALER BLACKJACK! Your insurance bet pays out {insurance_payout}. You now have {bankroll}.");
    }
  }


  if bankroll.is_zero() {
    bankroll += Money::from_major(1_000, iso::USD);
    println!("* Unfortunately, you've run out of money.");
    println!("* However, a portly gentleman in a sharp suit was watching you play your final hand.");
    println!("* He says \"I like your moxie, kiddo. Take this, and be a little more careful next time. This stuff doesn't grow on trees.\"");
    println!("* \"Oh, and always remember the name: MISTER GREEN!\"");
    println!("* The man hands you $1000.00.");
  }

  bankroll_file.set_len(0);
  bankroll_file.seek(SeekFrom::Start(0));
  bankroll_file.write_all(bankroll.to_string().as_bytes());
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
