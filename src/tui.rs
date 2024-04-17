use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::prelude::*;
use ratatui::{
    prelude::{CrosstermBackend, Terminal}, text::{Line, Text}, widgets::Paragraph
};
use std::io::{stdout, Result};

use crate::{blackjack::Casino, cards::Card, money::Money};

pub fn render_tui() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let casino = Casino::from_filesystem().unwrap();

    loop {
        terminal.draw(|frame| {
            let area = Rect::new(0, 0, frame.size().width, frame.size().height);
            frame.render_widget(&casino, area);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && key.code == KeyCode::Char('q')
                {
                    break;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

impl Widget for &Casino {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

    }
}

pub struct BankrollWidget(Money);

impl BankrollWidget {
    pub fn new(bankroll: Money) -> Self {
        Self(bankroll)
    }
}

impl Widget for &BankrollWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let money_width = (area.as_size().width - 9) as usize;
        Paragraph::new(format!("⛁ Money: {:>1$}", self.0, money_width)).render(area, buf);
    }
}


pub struct BetWidget(Money);

impl BetWidget {
    pub fn new(bet: Money) -> Self {
        Self(bet)
    }
}

impl Widget for &BetWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let money_width = (area.as_size().width - 7) as usize;
        Paragraph::new(format!("⛀ Bet: {:>1$}", self.0, money_width)).render(area, buf);
    }
}

pub struct CardWidget(Card);

impl CardWidget {
    pub fn new(card: Card) -> Self {
        Self(card)
    }
}

impl Widget for &CardWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let card = &self.0;

        Paragraph::new(Text::from(vec![
            Line::from("╭─────────╮"),
            Line::from(format!("│{:<9}│", card.value)),
            Line::from(format!("│{}        │", card.suit.symbol())),
            Line::from("│         │"),
            Line::from(format!("│    {}    │", card.suit.symbol())),
            Line::from("│         │"),
            Line::from(format!("│        {}│", card.suit.symbol())),
            Line::from(format!("│{:>9}│", card.value)),
            Line::from("╰─────────╯"),
        ])).render(area, buf);
    }
}

pub struct CardBackWidget;

impl CardBackWidget {
    pub fn new() -> Self { Self }
}

impl Widget for &CardBackWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        Paragraph::new(Text::from(vec![
            Line::from("╭─────────╮"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("╰─────────╯"),
        ])).render(area, buf);
    }
}

pub struct HorizontalCardStackWidget(Vec<Card>);

impl HorizontalCardStackWidget {
    pub fn new(cards: Vec<Card>) -> Self {
        Self(cards)
    }
}

impl Widget for &HorizontalCardStackWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let cards = &self.0;

        if cards.is_empty() {
            return;
        }

        let card = &cards[0];

        let top_card_lines = vec![
            format!("╭─────────╮"),
            format!("│{:<9}│", card.value),
            format!("│{}        │", card.suit.symbol()),
            format!("│         │"),
            format!("│    {}    │", card.suit.symbol()),
            format!("│         │"),
            format!("│        {}│", card.suit.symbol()),
            format!("│{:>9}│", card.value),
            format!("╰─────────╯"),
        ];


        let lines: Vec<Line> =
            cards[1..].iter().fold(top_card_lines, |lines, next_card| {
                vec![
                    format!("{}──╮", lines[0]),
                    format!("{}  │", lines[1]),
                    format!("{}  │", lines[2]),
                    format!("{}  │", lines[3]),
                    format!("{}  │", lines[4]),
                    format!("{}  │", lines[5]),
                    format!("{}{:>2}│", lines[6], next_card.suit.symbol()),
                    format!("{}{:>2}│", lines[7], next_card.value),
                    format!("{}──╯", lines[8]),
                ]
            }).iter().map(|l| Line::from(l.to_string())).collect();

        Paragraph::new(Text::from(lines)).render(area, buf);
    }
}

pub struct VerticalCardStackWidget(Vec<Card>);

impl VerticalCardStackWidget {
    pub fn new(cards: Vec<Card>) -> Self {
        Self(cards)
    }
}

impl Widget for &VerticalCardStackWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let cards = &self.0;

        if cards.is_empty() {
            return;
        }

        let card = &cards[0];

        let top_card_lines = vec![
            format!("╭─────────╮"),
            format!("│{:<9}│", card.value),
            format!("│{}        │", card.suit.symbol()),
            format!("│         │"),
            format!("│    {}    │", card.suit.symbol()),
            format!("│         │"),
            format!("│        {}│", card.suit.symbol()),
            format!("│{:>9}│", card.value),
            format!("╰─────────╯"),
        ];

        let lines: Vec<Line> =
            cards[1..].iter().fold(top_card_lines, |mut lines, next_card| {
                let mut lines_so_far = vec![
                    format!("╭─────────╮"),
                    format!("│{:<9}│", next_card.value),
                    format!("│{}        │", next_card.suit.symbol()),
                ];
                lines_so_far.append(&mut lines);
                lines_so_far
            }).iter().map(|l| Line::from(l.to_string())).collect();

        Paragraph::new(Text::from(lines)).render(area, buf);
    }
}
