# casino

![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/Cantido/casino/rust.yml)
[![Crates.io](https://img.shields.io/crates/d/casino)](https://crates.io/crates/casino)
![GitHub License](https://img.shields.io/github/license/Cantido/casino)

An entire casino built into your terminal.

## Installation

Install using `cargo`:

```console
$ cargo install casino
```

## Usage

Run `casino` for a selection of games, or `casino <game>` to start a specific game.

```console
$ casino blackjack
Your money: $1000.00
> How much will you bet? 100
Betting $100.00
* The dealer issues your cards.
Dealer's hand: 🂠 🃂
Your hand: 🃔 🂺  (14)
> What will you do? Stand
* Hole card revealed!
Dealer's hand: 🂻 🃂  (12)
* The dealer issues themself another card.
Dealer's hand: 🂻 🃂 🃕  (17)
* The hand is finished!
HOUSE WINS! You lose $100.00. You now have $900.00
```

You start with $1000.00, and if you ever hit $0.00, you are gifted another $1000.00.
This balance is persisted to your XDG data directory, along with the state of the deck of cards you're playing with.
You can modify this file if you want to break my heart and cheat at this innocent little terminal game.

Check your wallet balance with `casino balance`:

```console
$ casino balance
$1000.00
```

You can view your lifetime stats with `casino stats`:

```console
$ casino stats
Hands won.............................8
Hands lost...........................12
Hands tied............................1
Times hit bankruptcy..................0
Total money won.................1620.00
Total money lost................1240.00
Biggest win......................500.00
Biggest loss.....................500.00
Most money in the bank..........1555.00
```

Run `casino --help` for full usage instructions and documentation.

### Files

This program creates a few files, and respects the XDG Base Directory specification so as not to clutter up your home folder.

- `~/.config/casino/config.toml` - general app and game configuration
- `~/.local/share/casino/state.toml` - your current wallet balance and the state of the deck
- `~/.local/share/casino/stats.toml` - where statistics are collected for `casino stats`

## License

Copyright © 2023 Rosa Richter

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
