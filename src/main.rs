use std::fs;
use anyhow::Result;
use inquire::Select;
use clap::{Parser, Subcommand};
use casino::blackjack::Casino;
use casino::roulette::play_roulette;
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

      println!("Hands won...............{:.>15}", stats.hands_won);
      println!("Hands lost..............{:.>15}", stats.hands_lost);
      println!("Hands tied..............{:.>15}", stats.hands_push);
      println!("Times hit bankruptcy....{:.>15}", stats.times_bankrupted);
      println!("Total money won.........{:.>15}", stats.money_won);
      println!("Total money lost........{:.>15}", stats.money_lost);
      println!("Biggest win.............{:.>15}", stats.biggest_win);
      println!("Biggest loss............{:.>15}", stats.biggest_loss);
      println!("Most money in the bank..{:.>15}", stats.biggest_bankroll);
    },
    Some(Commands::Blackjack) => {
      let mut state = Casino::from_filesystem()?;
      state.play_blackjack()?;
    },
    Some(Commands::Roulette) => {
      play_roulette()?;
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
      let options = vec!["Blackjack"];

      let ans = Select::new("What would you like to play?", options).prompt();

      match ans {
        Ok("Blackjack") => {
          let mut state = Casino::from_filesystem()?;
          state.play_blackjack()?
        }
        Ok(_) => panic!("Unknown option"),
        Err(_) => panic!("Error fetching your choice"),
      }
    }
  }
  Ok(())
}

