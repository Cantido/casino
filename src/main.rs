use std::{fmt, fs, fs::OpenOptions, io, io::prelude::*, io::Read, io::SeekFrom, io::Write};
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

  let mut bankroll: i32 =
    if bankroll_buffer.is_empty() {
      bankroll_file.write_all(b"1000\n");
      1000
    } else {
      bankroll_buffer.trim().parse().unwrap()
    };


  println!("Your money: ${bankroll}.00");
  let mut bet: i32 = 0;

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
          bankroll -= bet;
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
      bankroll += bet;
      println!("DEALER BUST! You receive ${bet}.00. You now have ${bankroll}.00");
    } else if dealer_sum == player_sum {
      println!("PUSH! Nobody wins.");
    } else if dealer_sum > player_sum {
      bankroll -= bet;
      println!("YOU LOSE! You lose ${bet}.00. You now have ${bankroll}.00");
    } else {
      bankroll += bet;
      println!("YOU WIN! You receive ${bet}.00. You now have ${bankroll}.00");
    }
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
