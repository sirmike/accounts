use crate::enums::TransactionType;
use serde::{Deserialize, Deserializer};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct Transaction {
  pub r#type: TransactionType,
  pub client: u16,
  pub tx: u32,
  #[serde(deserialize_with = "string_to_decimal")]
  pub amount: Option<Decimal>
}

impl Transaction {
  pub fn default(client_id: u16, transaction_id: u32) -> Transaction {
    Transaction { r#type: TransactionType::Unknown, amount: None, client: client_id, tx: transaction_id }
  }
}

fn string_to_decimal<'a, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'a>,
{
  let s: &str = Deserialize::deserialize(deserializer)?;
  let value = s.trim();
  if value.is_empty() {
    return Ok(None);
  }

  let result = Decimal::from_str(value);
  match result {
    Ok(d) => { Ok(Some(d)) }
    Err(_) => { Err(serde::de::Error::custom(format!("Cannot parse {} to Decimal", value))) }
  }
}

