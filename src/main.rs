use std::fs;
use anyhow::Result;
use inquire::Select;
use clap::{Parser, Subcommand};
use casino::blackjack::Casino;
use casino::roulette::play_roulette;
use casino::slots::play_slots;
use casino::config::Config;

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
  /// Play a round of roulette
  Roulette,
  /// Spin a slot machine
  Slots,
  /// Show lifetime statistics
  Stats,
  /// Shuffle the persisted deck state
  Shuffle,
  /// Show currency balance
  Balance,
  /// Clears game state and statistics
  Reset,
}

fn main() -> Result<()> {
  let args = Args::parse();

  match &args.command {
    Some(Commands::Stats) => {
      let state = Casino::from_filesystem()?;
      let stats = state.stats;

      println!("Most money in the bank..{:.>15}", stats.biggest_bankroll);
      println!("Times hit bankruptcy....{:.>15}", stats.times_bankrupted);
      println!();

      println!("Blackjack");
      println!("  Hands won...............{:.>15}", stats.blackjack.hands_won);
      println!("  Hands lost..............{:.>15}", stats.blackjack.hands_lost);
      println!("  Hands tied..............{:.>15}", stats.blackjack.hands_push);
      println!("  Total money won.........{:.>15}", stats.blackjack.money_won);
      println!("  Total money lost........{:.>15}", stats.blackjack.money_lost);
      println!("  Biggest win.............{:.>15}", stats.blackjack.biggest_win);
      println!("  Biggest loss............{:.>15}", stats.blackjack.biggest_loss);

      println!();
      println!("Roulette");
      println!("  Spins won...............{:.>15}", stats.roulette.spins_won);
      println!("  Spins lost..............{:.>15}", stats.roulette.spins_lost);
      println!("  Total money won.........{:.>15}", stats.roulette.money_won);
      println!("  Total money lost........{:.>15}", stats.roulette.money_lost);
      println!("  Biggest win.............{:.>15}", stats.roulette.biggest_win);
      println!("  Biggest loss............{:.>15}", stats.roulette.biggest_loss);
    },
    Some(Commands::Blackjack) => {
      let mut state = Casino::from_filesystem()?;
      state.play_blackjack()?;
    },
    Some(Commands::Roulette) => {
      play_roulette()?;
    },
    Some(Commands::Slots) => {
      play_slots()?;
    },
    Some(Commands::Shuffle) => {
      let mut state = Casino::from_filesystem()?;
      state.shuffle_shoe();
      state.save();
    },
    Some(Commands::Balance) => {
      let state = Casino::from_filesystem()?;
      println!("{}", state.bankroll);
    }
    Some(Commands::Reset) => {
      let cfg_path = Config::default_path();
      let config = Config::from_path(&cfg_path)?;

      fs::remove_file(&config.save_path)?;
      fs::remove_file(&config.stats_path)?;
    }
    None => {
      let options = vec!["Blackjack", "Roulette"];

      let ans = Select::new("What would you like to play?", options).prompt();

      match ans {
        Ok("Blackjack") => {
          let mut state = Casino::from_filesystem()?;
          state.play_blackjack()?
        },
        Ok("Roulette") => {
          play_roulette()?;
        },
        Ok(_) => panic!("Unknown option"),
        Err(_) => panic!("Error fetching your choice"),
      }
    }
  }
  Ok(())
}

