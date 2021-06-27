use serde::ser::{Serialize, SerializeStruct, Serializer};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct Account {
  pub id: u16,
  pub available: Decimal,
  pub held: Decimal,
  pub locked: bool
}

impl Account {
  pub fn total(&self) -> Decimal {
    self.available + self.held
  }

  pub fn deposit(&mut self, value: Decimal) {
    assert!(value.is_sign_positive());
    if !self.locked {
      self.available += value;
    }
  }

  pub fn withdraw(&mut self, value: Decimal) {
    assert!(value.is_sign_positive());
    if self.available < value || self.locked {
      return
    }
    self.available -= value;
  }

  pub fn dispute(&mut self, value: Decimal) {
    assert!(value.is_sign_positive());
    if !self.locked {
      self.available -= value;
      self.held += value;
    }
  }

  pub fn resolve(&mut self, value: Decimal) {
    assert!(value.is_sign_positive());
    if !self.locked {
      self.available += value;
      self.held -= value;
    }
  }

  pub fn chargeback(&mut self, value: Decimal) {
    assert!(value.is_sign_positive());
    if !self.locked {
      self.held -= value;
      self.locked = true
    }
  }

  pub fn default(client_id: u16) -> Account {
      Account { available: dec!(0.0), held: dec!(0.0), id: client_id, locked: false }
  }
}

impl Serialize for Account {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
  {
    let mut s = serializer.serialize_struct("Account", 5)?;
    s.serialize_field("client", &self.id)?;
    s.serialize_field("available", &self.available)?;
    s.serialize_field("held", &self.held)?;
    s.serialize_field("total", &self.total())?;
    s.serialize_field("locked", &self.locked)?;
    s.end()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rust_decimal_macros::dec;

  #[test]
  fn deposit() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.deposit(dec!(3.0));
    assert_eq!(account.available, dec!(6.0));
  }

  #[test]
  fn deposit_if_locked() {
    let mut account = Account::default(1);
    account.locked = true;
    account.deposit(dec!(3.0));
    account.deposit(dec!(3.0));
    assert_eq!(account.available, dec!(0.0));
  }

  #[test]
  fn withdraw() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.withdraw(dec!(2.0));
    assert_eq!(account.available, dec!(1.0));
  }

  #[test]
  fn withdraw_if_locked() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.locked = true;
    account.withdraw(dec!(2.0));
    assert_eq!(account.available, dec!(3.0));
  }

  #[test]
  #[should_panic]
  fn withdraw_negative() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.withdraw(dec!(-2.0));
  }

  #[test]
  fn withdraw_too_much() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.withdraw(dec!(8.0));
    assert_eq!(account.available, dec!(3.0));
  }

  #[test]
  fn dispute() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.dispute(dec!(2.0));
    assert_eq!(account.available, dec!(1.0));
    assert_eq!(account.held, dec!(2.0));
  }

  #[test]
  #[should_panic]
  fn dispute_negative() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.dispute(dec!(-2.0));
  }

  #[test]
  fn dispute_locked() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.locked = true;
    account.dispute(dec!(2.0));
    assert_eq!(account.available, dec!(3.0));
    assert_eq!(account.held, dec!(0.0));
  }

  #[test]
  fn resolve() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.dispute(dec!(2.0));
    account.resolve(dec!(2.0));
    assert_eq!(account.available, dec!(3.0));
    assert_eq!(account.held, dec!(0.0));
  }

  #[test]
  #[should_panic]
  fn resolve_negative() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.resolve(dec!(-2.0));
  }

  #[test]
  fn resolve_locked() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.locked = true;
    account.resolve(dec!(2.0));
    assert_eq!(account.available, dec!(3.0));
    assert_eq!(account.held, dec!(0.0));
  }

  #[test]
  fn chargeback() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.dispute(dec!(2.0));
    account.chargeback(dec!(2.0));
    assert_eq!(account.available, dec!(1.0));
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.locked, true);
  }

  #[test]
  #[should_panic]
  fn chargeback_negative() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.chargeback(dec!(-2.0));
  }

  #[test]
  fn chargeback_locked() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.locked = true;
    account.chargeback(dec!(2.0));
    assert_eq!(account.available, dec!(3.0));
    assert_eq!(account.held, dec!(0.0));
  }

  #[test]
  fn total() {
    let mut account = Account::default(1);
    account.deposit(dec!(3.0));
    account.dispute(dec!(1.0));
    assert_eq!(account.total(), dec!(3.0));
  }
}
