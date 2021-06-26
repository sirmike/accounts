use super::enums;
use serde::Deserialize;
use rust_decimal::Decimal;

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Transaction {
  pub r#type: enums::TransactionType,
  pub client: u16,
  pub tx: u32,
  pub amount: f32
}

#[derive(Debug)]
pub struct Account {
  pub id: u16,
  pub available: Decimal,
  pub held: Decimal,
  pub locked: bool
}

impl Account {
  pub fn total(&self) -> Decimal {
    self.available - self.held
  }
}
