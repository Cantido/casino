# casino

An entire casino built into your terminal.

## Installation

Install using `cargo`:

```console
$ cargo install casino
```

## Usage

Run `casino blackjack` to play a game of blackjack.

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

Run `casino --help` for usage instructions.

## License

Copyright © 2023 Rosa Richter

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
