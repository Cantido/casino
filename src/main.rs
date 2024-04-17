use anyhow::{Context, Result};
use casino::blackjack::Casino;
use casino::config::Config;
use casino::roulette::play_roulette;
use casino::slots::play_slots;
use casino::craps::play_craps;
use clap::{Parser, Subcommand};
use inquire::Select;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Play a hand of blackjack
    Blackjack,
    /// Play a round of roulette
    Roulette,
    /// Spin a slot machine
    Slots,
    /// Play a game of craps
    Craps,
    /// Show lifetime statistics
    Stats,
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
            println!(
                "  Hands won...............{:.>15}",
                stats.blackjack.hands_won
            );
            println!(
                "  Hands lost..............{:.>15}",
                stats.blackjack.hands_lost
            );
            println!(
                "  Hands tied..............{:.>15}",
                stats.blackjack.hands_push
            );
            println!(
                "  Total money won.........{:.>15}",
                stats.blackjack.money_won
            );
            println!(
                "  Total money lost........{:.>15}",
                stats.blackjack.money_lost
            );
            println!(
                "  Biggest win.............{:.>15}",
                stats.blackjack.biggest_win
            );
            println!(
                "  Biggest loss............{:.>15}",
                stats.blackjack.biggest_loss
            );

            println!();
            println!("Roulette");
            println!(
                "  Spins won...............{:.>15}",
                stats.roulette.spins_won
            );
            println!(
                "  Spins lost..............{:.>15}",
                stats.roulette.spins_lost
            );
            println!(
                "  Total money won.........{:.>15}",
                stats.roulette.money_won
            );
            println!(
                "  Total money lost........{:.>15}",
                stats.roulette.money_lost
            );
            println!(
                "  Biggest win.............{:.>15}",
                stats.roulette.biggest_win
            );
            println!(
                "  Biggest loss............{:.>15}",
                stats.roulette.biggest_loss
            );

            println!();
            println!("Slots");
            println!("  Total pulls.............{:.>15}", stats.slots.total_pulls);
            println!("  Total money spent.......{:.>15}", stats.slots.money_spent);
            println!("  Total money won.........{:.>15}", stats.slots.money_won);
            println!(
                "  Biggest jackpot.........{:.>15}",
                stats.slots.biggest_jackpot
            );
        }
        Some(Commands::Blackjack) => {
            let mut state = Casino::from_filesystem()
                .with_context(|| "Failed to load Casino configuration from filesystem.")?;

            state.play_blackjack()
                .with_context(|| "Failed to finish a game of Blackjack")?;
        }
        Some(Commands::Roulette) => {
            play_roulette()?;
        }
        Some(Commands::Slots) => {
            play_slots()?;
        }
        Some(Commands::Craps) => {
            play_craps()?;
        }
        Some(Commands::Balance) => {
            let state = Casino::from_filesystem()?;
            println!("{}", state.bankroll);
        }
        Some(Commands::Reset) => {
            let cfg_path = Config::default_path();
            let config = Config::from_path(&cfg_path)?;

            if config.save_path.try_exists()? {
                println!("Deleting save file at {}", config.save_path.display());
                fs::remove_file(&config.save_path)
                    .with_context(|| "Failed to remove save file")?;
            }
            if config.stats_path.try_exists()? {
                println!("Deleting stats file at {}", config.stats_path.display());
                fs::remove_file(&config.stats_path)
                    .with_context(|| "Failed to remove stats file")?;
            }
        }
        None => {
            let options = vec!["Blackjack", "Roulette", "Slots", "Craps"];

            let ans = Select::new("What would you like to play?", options).prompt();

            match ans {
                Ok("Blackjack") => {
                    let mut state = Casino::from_filesystem()?;
                    state.play_blackjack()?
                },
                Ok("Roulette") => {
                    play_roulette()?;
                },
                Ok("Slots") => {
                    play_slots()?;
                },
                Ok("Craps") => {
                    play_craps()?;
                },
                Ok(_) => panic!("Unknown option"),
                Err(_) => panic!("Error fetching your choice"),
            }
        }
    }
    Ok(())
}
