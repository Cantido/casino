use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use itertools::Itertools;
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::{
    prelude::{CrosstermBackend, Terminal}, text::{Line, Text}, widgets::Paragraph
};
use std::{io::{stdout, Result}, time::Duration};

use crate::{blackjack::{Casino, Hand}, cards::Card, money::Money};

pub fn render_tui() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut model = Model::new();

    while model.state != BlackjackState::Stopped {
        terminal.draw(|frame| view(&mut model, frame))?;

        let mut current_msg = handle_event(&model)?;

        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[derive(PartialEq)]
enum Message {
    Quit,
    Left,
    Right,
    Increment,
    Decrement,
    Select,
    Hit,
    Stand,
}

struct Model {
    pub casino: Casino,
    pub state: BlackjackState,
    pub bet_digit_selector: u32,
    pub player_action_list_state: ListState,
}
impl Model {
    fn new() -> Self {
        Self {
            casino: Casino::from_filesystem().unwrap(),
            state: BlackjackState::PlacingBet,
            bet_digit_selector: 0u32,
            player_action_list_state: ListState::default().with_selected(Some(0)),
        }
    }
}

fn view(model: &mut Model, frame: &mut Frame) {
    let area = Rect::new(0, 0, frame.size().width, frame.size().height);

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(25),
            Constraint::Length(55),
        ])
        .split(area);

    let cards_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(10),
            Constraint::Length(14),
        ])
        .split(main_layout[1]);

    frame.render_widget(
       &DealerHandWidget::new(&model.casino.blackjack.dealer_hand),
       cards_layout[0],
    );

    let player_hands_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(11)].repeat(model.casino.blackjack.player_hands.len()))
        .split(cards_layout[1]);

    for (i, hand) in model.casino.blackjack.player_hands.iter().enumerate() {
        let hand_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(player_hands_layout[i]);

        let hand_str =
            if hand.is_bust() {
                "BUST".to_string()
            } else  {
                hand.blackjack_sum().to_string()
            };

        frame.render_widget(
            Paragraph::new(Line::from(hand_str)).centered().reversed(),
            hand_layout[0]
        );

        frame.render_widget(
            &VerticalCardStackWidget::new(&hand.cards),
            hand_layout[1]
        );
    }


    let data_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(9),
            Constraint::Length(2),
            Constraint::Length(13),
        ])
        .split(main_layout[0]);

    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::from("Dealer "),
            Line::from("You "),
        ])).right_aligned(),
        data_layout[1]
    );


    frame.render_widget(
        &GameStateWidget::new(&model.casino, &model.state),
        data_layout[0],
    );

    if model.state == BlackjackState::PlacingBet {
        let block = Block::default().title("Place bet").borders(Borders::ALL);

        frame.render_stateful_widget(PlaceBetWidget::new(&model.casino.bankroll, &model.casino.blackjack.bet), block.inner(data_layout[2]), &mut model.bet_digit_selector);
        frame.render_widget(block, data_layout[2]);

    } else if model.state == BlackjackState::GameOver {
        let block = Block::default().title("Actions").borders(Borders::ALL);

        frame.render_widget(
            FillWidget::new('╱'),
            block.inner(data_layout[2])
        );

        let vert_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Fill(1),
            ])
            .split(block.inner(data_layout[2]));

        let horiz_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(15),
                Constraint::Fill(1),
            ])
            .split(vert_layout[1]);

        let payout = model.casino.blackjack.payout();

        let game_over_message =
            if payout.is_sign_positive() {
                Paragraph::new(Text::from(vec![
                  Line::from("Game over!"),
                  Line::from(""),
                  Line::from("You receive"),
                  Line::from(payout.to_string()),
                ])).centered()
            } else {
                Paragraph::new(Text::from(vec![
                  Line::from("Game over!"),
                  Line::from(""),
                  Line::from("You lose"),
                  Line::from(payout.abs().to_string()),
                ])).centered()
            };

        frame.render_widget(Clear, horiz_layout[1]);
        frame.render_widget(
            game_over_message,
            horiz_layout[1],
        );

        frame.render_widget(block, data_layout[2]);
    } else {
        frame.render_stateful_widget(
            &List::new(vec!["Hit", "Stand", "Surrender"])
                .block(Block::default().title("Actions").borders(Borders::ALL))
                .highlight_spacing(HighlightSpacing::Always)
                .highlight_style(Style::new().reversed())
                .highlight_symbol("➤"),
            data_layout[2],
            &mut model.player_action_list_state,
        );
    }

}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    if msg == Message::Quit {
        model.state = BlackjackState::Stopped;
        return None;
    }

    match model.state {
        BlackjackState::PlacingBet => {
            match msg {
                Message::Left => {
                    if model.bet_digit_selector < model.casino.bankroll.major_digit_count() {
                        model.bet_digit_selector += 1;
                    }
                    None
                },
                Message::Right => {
                    if model.bet_digit_selector > 0 {
                        model.bet_digit_selector -= 1;
                    }
                    None
                },
                Message::Increment => {
                    let amount_to_add = Money::from_major(10_i32.pow(model.bet_digit_selector).into());

                    if model.casino.blackjack.bet + amount_to_add <= model.casino.bankroll {
                        model.casino.blackjack.bet += amount_to_add;
                    }
                    None
                }
                Message::Decrement => {
                    let amount_to_subtract = Money::from_major(10_i32.pow(model.bet_digit_selector).into());

                    if model.casino.blackjack.bet - amount_to_subtract >= Money::ZERO {
                        model.casino.blackjack.bet -= amount_to_subtract;
                    }
                    None
                },
                Message::Select => {
                    if model.casino.blackjack.bet > Money::ZERO {
                        model.casino.blackjack.initial_deal();
                        model.state = BlackjackState::PlayerTurn;
                    }
                    None
                }
                _ => None
            }
        }
        BlackjackState::PlayerTurn => {
            match msg {
                Message::Increment => {
                    if let Some(n) = model.player_action_list_state.selected() {
                        if n > 0 {
                            model.player_action_list_state.select(Some(n - 1));
                        }
                    }
                    None
                }
                Message::Decrement => {
                    if let Some(n) = model.player_action_list_state.selected() {
                        if n < 2 {
                            model.player_action_list_state.select(Some(n + 1));
                        }
                    }
                    None
                }
                Message::Select => {
                    match model.player_action_list_state.selected().unwrap() {
                        0 => Some(Message::Hit),
                        1 => Some(Message::Stand),
                        2 => Some(Message::Quit),
                        _ => panic!("Unknown list state"),
                    }
                }
                Message::Hit => {
                    model.casino.blackjack.hit();

                    if model.casino.blackjack.player_hands.iter().all(|h| h.is_bust()) {
                        model.state = BlackjackState::GameOver;
                    }
                    None
                }
                Message::Stand => {
                    model.casino.blackjack.stand();
                    model.casino.blackjack.reveal_hole_card();
                    model.state = BlackjackState::DealerTurn;

                    while model.casino.blackjack.dealer_hand.blackjack_sum() < 17 {
                        model.casino.blackjack.card_to_dealer();
                    }

                    if model.casino.blackjack.dealer_hand.is_bust() {
                        model.state = BlackjackState::GameOver;
                    }

                    None
                }
                _ => None
            }
        }
        BlackjackState::DealerTurn => {
            match msg {
                Message::Select => {
                    model.state = BlackjackState::GameOver;
                    return None;
                }
                _ => None
            }
        }
        BlackjackState::GameOver => {
            None
        }
        _ => unimplemented!(),
    }
}

fn handle_event(model: &Model) -> Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(&model.state, key));
            }
        }
    }
    Ok(None)
}

fn handle_key(state: &BlackjackState, key: event::KeyEvent) -> Option<Message> {
    if key.code == KeyCode::Char('q') {
        return Some(Message::Quit);
    }
    match state {
        BlackjackState::PlacingBet => {
            match key.code {
                KeyCode::Char('a') | KeyCode::Left => Some(Message::Left),
                KeyCode::Char('d') | KeyCode::Right => Some(Message::Right),
                KeyCode::Char('w') | KeyCode::Up => Some(Message::Increment),
                KeyCode::Char('s') | KeyCode::Down => Some(Message::Decrement),
                KeyCode::Char(' ') | KeyCode::Enter => Some(Message::Select),
                _ => None,
            }
        }
        BlackjackState::PlayerTurn => {
            match key.code {
                KeyCode::Char('w') | KeyCode::Up => Some(Message::Increment),
                KeyCode::Char('s') | KeyCode::Down => Some(Message::Decrement),
                KeyCode::Char(' ') | KeyCode::Enter => Some(Message::Select),
                _ => None,
            }
        }
        BlackjackState::DealerTurn => {
            match key.code {
                KeyCode::Char(' ') => Some(Message::Select),
                _ => None,
            }
        }
        BlackjackState::GameOver => {
            Some(Message::Quit)
        }
        _ => unimplemented!(),
    }
}



#[derive(PartialEq)]
enum BlackjackState {
    PlacingBet,
    PlayerTurn,
    DealerTurn,
    GameOver,
    Stopped,
}

struct GameStateWidget<'a>(&'a Casino, &'a BlackjackState);

impl<'a> GameStateWidget<'a> {
    pub fn new(casino: &'a Casino, state: &'a BlackjackState) -> Self {
        Self(casino, state)
    }
}

impl Widget for &GameStateWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {

        let block = Block::default()
            .title_top("Game")
            .borders(Borders::ALL);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Fill(1),
            ]).split(block.inner(area));

        block.render(area, buf);

        BankrollWidget::new(self.0.bankroll)
            .render(layout[0], buf);
        BetWidget::new(self.0.blackjack.bet)
            .render(layout[1], buf);


        let rows = [
            Row::new(vec![
                Cell::from("Double-down"),
                Cell::from(stylized_bool(self.0.blackjack.doubling_down).into_right_aligned_line()),
            ]),
            Row::new(vec![
                Cell::from("Split"),
                Cell::from(stylized_bool(self.0.blackjack.splitting).into_right_aligned_line()),
            ]),
            Row::new(vec![
                Cell::from("Insurance"),
                Cell::from(stylized_bool(self.0.blackjack.insurance).into_right_aligned_line()),
            ]),
        ];

        let widths = [
            Constraint::Fill(1),
            Constraint::Fill(1),
        ];

        let table = Table::new(rows, widths)
            .block(Block::default()
            );

        ratatui::widgets::Widget::render(table, layout[2], buf);
    }
}

fn stylized_bool(b: bool) -> Span<'static> {
    if b {
        Span::from("true").yellow().bold()
    } else {
        Span::from("false").dim()
    }
}

struct FillWidget(char);

impl FillWidget {
    pub fn new(fill_char: char) -> Self {
        Self(fill_char)
    }
}

impl Widget for FillWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let line = "╱".repeat(area.width.into());
        let mut lines = vec![];

        for _i in 0..area.height {
            lines.push(Line::from(line.clone()));
        }

        Paragraph::new(Text::from(lines))
            .render(area, buf);
    }
}

pub struct PlaceBetWidget<'a>{
    bankroll: &'a Money,
    bet: &'a Money,
}

impl<'a> PlaceBetWidget<'a> {
    pub fn new(bankroll: &'a Money, bet: &'a Money) -> Self {
        Self { bankroll, bet }
    }
}

impl StatefulWidget for PlaceBetWidget<'_> {
    type State = u32;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        BankrollWidget::new(*self.bankroll)
            .render(main_layout[0], buf);
        BetWidget::new(*self.bet)
            .render(main_layout[1], buf);

        let selector_u16: u16 = state.clone().try_into().unwrap();
        let selector_column = main_layout[2].as_size().width - (3 + selector_u16);

        let selector_pos = Rect::new(selector_column.try_into().unwrap(), main_layout[2].y, 1, 1);

        Paragraph::new("⌃").render(selector_pos, buf);
    }
}


pub struct DealerHandWidget<'a>(&'a Hand);

impl<'a> DealerHandWidget<'a> {
    pub fn new(hand: &'a Hand) -> Self {
        Self(hand)
    }
}

impl Widget for &DealerHandWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {

        if self.0.cards.is_empty() {
            return;
        }

        let max_card_count = area.x / 11;

        if self.0.cards.len() <= max_card_count.into() {

            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![ Constraint::Length(11) ].repeat(self.0.cards.len()))
                .split(area);

            assert!(self.0.hidden_count <= 1);

            let hole_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(9),
                    Constraint::Length(1)
                ])
                .split(layout[0]);

            let dealer_sum =
                if self.0.hidden_count > 0 {
                    "?".to_string()
                } else if self.0.is_bust() {
                    "BUST".to_string()
                } else {
                    self.0.blackjack_sum().to_string()
                };

            Paragraph::new(Span::from(dealer_sum).into_centered_line())
                .reversed()
                .render(hole_layout[1], buf);

            for (i, card) in self.0.cards.iter().enumerate() {
                if i < self.0.hidden_count {
                    CardBackWidget::new()
                        .render(layout[i], buf);
                } else {
                    CardWidget::new(card)
                        .render(layout[i], buf);
                }
            }
        } else {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                     Constraint::Length(11),
                     Constraint::Fill(1),
                ])
                .split(area);

            let hole_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(9),
                    Constraint::Length(1)
                ])
                .split(layout[0]);

            let dealer_sum =
                if self.0.hidden_count > 0 {
                    "?".to_string()
                } else {
                    self.0.blackjack_sum().to_string()
                };

            Paragraph::new(Span::from(dealer_sum))
                .reversed()
                .centered()
                .render(hole_layout[1], buf);

            if self.0.hidden_count == 1 {
                CardBackWidget::new()
                    .render(layout[0], buf);

                HorizontalCardStackWidget::new(&self.0.cards[1..])
                    .render(layout[1], buf);
            } else if self.0.hidden_count == 0 {
                CardWidget::new(&self.0.cards[0])
                    .render(layout[0], buf);

                HorizontalCardStackWidget::new(&self.0.cards[1..])
                    .render(layout[1], buf);
            } else {
                unimplemented!();
            }
        }
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

pub struct CardWidget<'a>(&'a Card);

impl<'a> CardWidget<'a> {
    pub fn new(card: &'a Card) -> Self {
        Self(card)
    }
}

impl Widget for &CardWidget<'_> {
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
            Line::from("│╔═══════╗│"),
            Line::from("│║       ║│"),
            Line::from("│║       ║│"),
            Line::from("│║       ║│"),
            Line::from("│║       ║│"),
            Line::from("│║       ║│"),
            Line::from("│╚═══════╝│"),
            Line::from("╰─────────╯"),
        ])).render(area, buf);
    }
}

pub struct HorizontalCardStackWidget<'a>(&'a [Card]);

impl<'a> HorizontalCardStackWidget<'a> {
    pub fn new(cards: &'a [Card]) -> Self {
        Self(cards)
    }
}

impl Widget for &HorizontalCardStackWidget<'_> {
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
            cards.iter().dropping(1).fold(top_card_lines, |lines, next_card| {
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

pub struct VerticalCardStackWidget<'a>(&'a [Card]);

impl<'a> VerticalCardStackWidget<'a> {
    pub fn new(cards: &'a [Card]) -> Self {
        Self(cards)
    }
}

impl Widget for &VerticalCardStackWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let cards = &self.0;

        if cards.is_empty() {
            return;
        }

        let card = &cards[cards.len() - 1];

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
            cards.iter().rev().dropping(1).fold(top_card_lines, |mut lines, next_card| {
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
