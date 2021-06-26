use std::error::Error;
use csv::ReaderBuilder;
use csv::DeserializeRecordsIter;
use csv::Trim;
use std::collections::HashMap;

use super::structs::Account;
use super::structs::Transaction;
use super::enums::TransactionType;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;

pub fn read(path: &str) -> Result<HashMap<u16, Account>, Box<dyn Error>> {
  let mut result: HashMap<u16, Account> = HashMap::new();
  let mut reader = ReaderBuilder::new().
    has_headers(true).
    trim(Trim::All).
    from_path(path)?;
  let iter: DeserializeRecordsIter<'_, std::fs::File, Transaction> = reader.deserialize();
  for data in iter {
    match data {
      Ok(val) => {
        let account = result.entry(val.client).or_insert(
            Account {
              id: val.client,
              available: dec!(0),
              held: dec!(0),
              locked: false
            }
          );
        match val.r#type {
          TransactionType::Deposit => {
            let deposit: Decimal = FromPrimitive::from_f32(val.amount).unwrap();
            account.available += deposit;
          },
          TransactionType::Withdrawal => {
            let withdrawal: Decimal = FromPrimitive::from_f32(val.amount).unwrap();
            account.available -= withdrawal;
          }
          _ => {}
        }
      },
      Err(_err) => {
        return Err(Box::new(_err))
      }
    }
  }
  Ok(result)
}
