use core::fmt;
use std::{io::{stdout, Write}, thread::sleep, time::Duration};

use anyhow::{ensure, Result};
use colored::*;
use crossterm::{cursor, terminal, QueueableCommand};
use inquire::{Select, Text};
use rand::{thread_rng, Rng};

use crate::{blackjack::Casino, money::Money};

pub fn play_craps() -> Result<()> {
    let mut casino = Casino::from_filesystem()?;
    // Pass Line/Don't Pass bets
    // - pay 1:1
    // - 2 or 3 - Pass bets lose, Don't Pass bets win
    // - 7 or 11 - Pass bets win, Don't Pass bets lose
    // - 12 - Pass & Don't Pass bets are a push

    let pass_bet = prompt_for_line_bets(&casino.bankroll);

    // Roll the come-out roll - becomes the "point"
    //
    // Remaining bets become available
    // - Come/Don't Come Bet - same as Pass/Don't Pass, pays 1:1
    // - Field Bet (SINGLE ROLL) - pays 1:1 on [3, 9, 10, 11], 2:1 on 2, 3:1 on 12
    // - Free Odds - bet 7 will be rolled before another number
    //   - pays 1:2 on [4, 10]
    //   - pays 2:3 on [5, 9]
    //   - pays 5:6 on [6, 8]
    // - Place bets - bet another number will be rolled before 7
    //   - 9:5 for [4, 10]
    //   - 7:5 for [7, 9]
    //   - 7:6 for [6, 8]
    // - Buy bets - same as Place bets but casino pays "true odds" but takes 5% commission
    //   - 2:1 on [4, 10]
    //   - 6:5 on [6, 8]
    //   - 3:2 on [5, 9]
    // - Big Six/Eight - pays out 1:1 if a [6, 8] is thrown before 7
    // - Hardway bets - bet that both dice show the same number
    //   - 9:1 on 6 or 8
    //   - 7:1 on 4 or 10
    // - 2 or 12 (SINGLE ROLL) - pays 30:1 on a 2 or 12 on the next roll
    // - 3 or 11 (SINGLE ROLL) - pays 15:1 on a 3 or 11 on the next roll
    // - any 7 (SINGLE ROLL) - pays 4:1 if the next roll is 7
    // - any craps (SINGLE ROLL) - pays 7:1 if next roll is 2, 3, or 12
    //
    // Roll again:
    // - 7 - you lose
    // - point - you win
    // - any other number - roll again


    let point = animate_roll();

    if pass_bet.is_win(point) {
        println!("A {}! Your {} bet wins!", point, pass_bet.kind);
        let payout = pass_bet.payout(point);
        casino.add_bankroll(payout);
        println!("You receive {}. You now have {}", payout, casino.bankroll);

        casino.save();

        return Ok(());
    }

    if pass_bet.is_lose(point) {
        println!("A {}! Your {} bet loses!", point, pass_bet.kind);
        casino.subtract_bankroll(pass_bet.amount)?;
        println!("You lose {}. You now have {}", pass_bet.amount, casino.bankroll);

        casino.save();

        return Ok(());
    }

    let mut bets = vec![];

    loop {
        let loop_options = vec!["Roll the dice again", "Place another bet"];

        let loop_selection = Select::new("What do you do?", loop_options).prompt().unwrap();

        match loop_selection {
            "Place another bet" => {
                let bet_kind_options = vec![BetKind::Come, BetKind::DontCome, BetKind::Field];

                let come_bet_kind = Select::new("What kind of bet would you like to place?", bet_kind_options).prompt().unwrap();

                loop {
                    let bet_result = Text::new("How much will you bet?").prompt();

                    match bet_result {
                        Ok(bet_text) => {
                            let bet = bet_text.trim().parse::<Money>().unwrap();
                            if bet <= casino.bankroll {
                                let bet = Bet::new(come_bet_kind, bet);
                                bets.push(bet);
                                break;

                            } else {
                                println!("You can't bet that amount, try again.");
                            }
                        }
                        Err(_) => panic!("Error getting your answer."),
                    }
                }
            }
            "Roll the dice again" => {}
            _ => panic!("Unknown selection {}", loop_selection)
        }


        let roll = animate_roll();

        let (wins, rest): (Vec<Bet>, Vec<Bet>) = bets.into_iter().partition(|b| b.is_win(roll));
        let (losses, rest): (Vec<Bet>, Vec<Bet>) = rest.into_iter().partition(|b| b.is_lose(roll));
        let (expired, rest): (Vec<Bet>, Vec<Bet>) = rest.into_iter().partition(|b| b.duration() == BetDuration::SingleRoll);

        for bet in wins.iter() {
            println!("A {}! Your {} bet wins!", roll, bet.kind);
            let payout = bet.payout(roll);
            casino.add_bankroll(payout);
            println!("You receive {}. You now have {}", payout, casino.bankroll);
        }

        for bet in losses.iter() {
            println!("A {}! Your {} bet loses!", roll, bet.kind);
            casino.subtract_bankroll(bet.amount)?;
            println!("You lose {}. You now have {}", bet.amount, casino.bankroll);
        }

        for bet in expired.iter() {
            println!("Your {} bet expires!", bet.kind);
            casino.subtract_bankroll(bet.amount)?;
            println!("You lose {}. You now have {}", bet.amount, casino.bankroll);
        }

        bets = rest;

        if roll == point {
            println!("A {}! The round is over!", roll);

            if !bets.is_empty() {
                println!("Remaining bets lose:");

                let mut lost_amount = Money::ZERO;

                for bet in bets.iter() {
                    println!("- Your {} bet for {} loses.", bet.kind, bet.amount);
                    lost_amount += bet.amount;
                }

                if lost_amount > Money::ZERO {
                    casino.bankroll -= lost_amount;
                    println!("You lose {}. You now have {}", lost_amount, casino.bankroll);
                    casino.save();
                }
            }

            break;
        }
    }

    casino.save();

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum BetDuration {
    MultiRoll,
    SingleRoll,
}

enum BetKind {
    PassLine,
    DontPass,
    Come,
    DontCome,
    Field,
}

impl fmt::Display for BetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PassLine => write!(f, "Pass Line"),
            Self::DontPass => write!(f, "Don't Pass"),
            Self::Come => write!(f, "Come"),
            Self::DontCome => write!(f, "Don't Come"),
            Self::Field => write!(f, "Field"),
        }
    }
}

struct Bet {
    pub kind: BetKind,
    pub amount: Money,
}

impl Bet {
    pub fn new(kind: BetKind, amount: Money) -> Self {
        Self { kind, amount }
    }

    pub fn is_win(&self, roll: u8) -> bool {
        match self.kind {
            BetKind::PassLine => vec![7, 11].contains(&roll),
            BetKind::DontPass => vec![2, 3, 12].contains(&roll),
            BetKind::Come => vec![7, 11].contains(&roll),
            BetKind::DontCome => vec![2, 3, 12].contains(&roll),
            BetKind::Field => vec![2, 3, 4, 9, 10, 11, 12].contains(&roll),
        }
    }

    pub fn is_lose(&self, roll: u8) -> bool {
        match self.kind {
            BetKind::PassLine => vec![2, 3, 12].contains(&roll),
            BetKind::DontPass => vec![7, 11].contains(&roll),
            BetKind::Come => vec![2, 3, 12].contains(&roll),
            BetKind::DontCome => vec![7, 11].contains(&roll),
            BetKind::Field => vec![5, 6, 7, 8].contains(&roll),
        }
    }

    pub fn payout(&self, roll: u8) -> Money {
        match self.kind {
            BetKind::PassLine => self.amount,
            BetKind::DontPass => self.amount,
            BetKind::Come => self.amount,
            BetKind::DontCome => self.amount,
            BetKind::Field => {
                match roll {
                    2 | 12 => self.amount * 2i64,
                    _ => self.amount,
                }
            },
        }
    }

    pub fn duration(&self) -> BetDuration {
        match self.kind {
            BetKind::PassLine => BetDuration::MultiRoll,
            BetKind::DontPass => BetDuration::MultiRoll,
            BetKind::Come => BetDuration::MultiRoll,
            BetKind::DontCome => BetDuration::MultiRoll,
            BetKind::Field => BetDuration::SingleRoll,
        }
    }
}

fn prompt_for_line_bets(max: &Money) -> Bet {
    let bet_kind_options = vec![BetKind::PassLine, BetKind::DontPass];

    let pass_bet_kind = Select::new("What kind of bet would you like to place?", bet_kind_options).prompt().unwrap();

    loop {
        let bet_result = Text::new("How much will you bet?").prompt();

        match bet_result {
            Ok(bet_text) => {
                let bet = bet_text.trim().parse::<Money>().unwrap();
                if bet <= *max {
                    return Bet::new(pass_bet_kind, bet);
                } else {
                    println!("You can't bet that amount, try again.");
                }
            }
            Err(_) => panic!("Error getting your answer."),
        }
    }
}

pub struct Die(u8);

impl Die {
    pub fn new(num: u8) -> Result<Self> {
        ensure!(num >= 1 && num <= 6, "Number outside of die range");

        Ok(Self(num))
    }

    pub fn roll() -> Self {
        let mut rng = thread_rng();
        let num = rng.gen_range(1..=6);

        Self(num)
    }
}

impl Die {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for Die {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            1 => write!(f, "⚀"),
            2 => write!(f, "⚁"),
            3 => write!(f, "⚂"),
            4 => write!(f, "⚃"),
            5 => write!(f, "⚄"),
            6 => write!(f, "⚅"),
            _ => panic!("Bad die number"),
        }
    }
}

fn animate_roll() -> u8 {
    println!("{}", "* You throw the dice.".dimmed());
    println!();

    let mut d1 = Die::roll();
    let mut d2 = Die::roll();

    let mut rng = thread_rng();
    let mut position = 0.0;
    let mut velocity = rng.gen_range(20.0..40.0);
    let accel = rng.gen_range(-30.0..-15.0);

    let mut stdout = stdout();

    while velocity > 0.0 {
        if position >= 1.0 {
            d1 = Die::roll();
            d2 = Die::roll();
            position -= 1.0;
        }
        stdout.queue(cursor::SavePosition).unwrap();
        stdout.write_all(format!("\t{}{}", d1, d2).as_bytes()).unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();

        sleep(Duration::from_millis(16));

        velocity += accel * (16.0 / 1000.0);
        position += velocity * (16.0 / 1000.0);

        stdout.queue(cursor::RestorePosition).unwrap();
        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
    }

    stdout.write_all(format!("\t{}{}", d1, d2).as_bytes()).unwrap();
    sleep(Duration::from_millis(1_000));
    println!();
    println!();

    d1.as_u8() + d2.as_u8()
}


