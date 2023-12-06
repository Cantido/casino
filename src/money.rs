use std::ops::{Add, AddAssign, Div, Mul, MulAssign, SubAssign};
use std::str::FromStr;
use num::rational::Ratio;
use rust_decimal::Decimal;
use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Money(
  #[serde(with = "rust_decimal::serde::str")]
  Decimal
);

#[derive(Debug, PartialEq, Eq)]
pub struct ParseMoneyError;

impl Money {
  pub const ZERO: Money = Money(Decimal::ZERO);

  pub fn from_major(dollars: i64) -> Self {
    Money(Decimal::new(dollars, 0))
  }

  pub fn is_zero(&self) -> bool {
    self.0.is_zero()
  }

  pub fn is_sign_positive(&self) -> bool {
    self.0.is_sign_positive()
  }
}

impl From<Decimal> for Money {
  fn from(dec: Decimal) -> Self {
    Self(dec)
  }
}

impl From<Money> for Decimal {
  fn from(money: Money) -> Self {
    money.0
  }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.2}", self.0)
    }
}

impl FromStr for Money {
  type Err = ParseMoneyError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Decimal::from_str(s).map(|dec| Self(dec.round_dp(2))).map_err(|_| ParseMoneyError)
  }
}

impl Add for Money {
  type Output = Money;

  fn add(self, other: Money) -> Self::Output {
    Money(self.0.add(other.0))
  }
}

impl AddAssign for Money {
  fn add_assign(&mut self, other: Money) {
    self.0.add_assign(other.0);
  }
}

impl Div<i64> for Money {
  type Output = Money;

  fn div(self, other: i64) -> Self::Output {
    Money(self.0.div(Decimal::new(other, 0)).round_dp(2))
  }
}

impl SubAssign for Money {
  fn sub_assign(&mut self, other: Money) {
    self.0.sub_assign(other.0);
  }
}

impl Mul<i64> for Money {
  type Output = Money;

  fn mul(self, other: i64) -> Money {
    Money(self.0.mul(Decimal::new(other, 0)))
  }
}

impl Mul<Ratio<i64>> for Money {
  type Output = Money;

  fn mul(self, ratio: Ratio<i64>) -> Money {
    self * *ratio.numer() / *ratio.denom()
  }
}

impl MulAssign<i64> for Money {
  fn mul_assign(&mut self, other: i64) {
    self.0.mul_assign(Decimal::new(other, 0));
  }
}
