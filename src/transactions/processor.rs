use csv::DeserializeRecordsIter;
use csv::{Reader, Error, Position};

use std::collections::HashMap;
use std::io;

use crate::structs::Transaction;
use crate::structs::Account;
use crate::enums::TransactionType;

pub fn process<R: io::Read + io::Seek>(data_reader: &mut Reader<R>) -> Result<HashMap<u16, Account>, Error> {
  let mut disputes = read_disputes(data_reader)?;

  // Rewind and skip headers
  data_reader.seek(Position::new())?;
  let mut header = csv::StringRecord::new();
  data_reader.read_record(&mut header)?;

  read_data(data_reader, &mut disputes)
}

fn read_disputes<R: io::Read + io::Seek>(data_reader: &mut Reader<R>) -> Result<HashMap<u32, Transaction>, Error> {
  let mut result = HashMap::new();
  let iter: DeserializeRecordsIter<'_, R, Transaction> = data_reader.deserialize();
  for data in iter {
    let tx = data?;
    if tx.r#type == TransactionType::Dispute {
      result.insert(
        tx.tx,
        Transaction::default(tx.client, tx.tx)
      );
    }
  }

  Ok(result)
}

fn read_data<R: io::Read + io::Seek>(data_reader: &mut Reader<R>, disputes: &mut HashMap<u32, Transaction>) -> Result<HashMap<u16, Account>, Error> {
  let mut accounts: HashMap<u16, Account> = HashMap::new();

  let iter: DeserializeRecordsIter<'_, R, Transaction> = data_reader.deserialize();
  for data in iter {
    let tx_data = data?;
    let account = accounts.entry(tx_data.client).or_insert(Account::default(tx_data.client));

    try_to_fill_dispute_with_transaction(&tx_data, disputes);
    commit_transaction(account, &tx_data, disputes);
  }
  Ok(accounts)
}

fn try_to_fill_dispute_with_transaction(transaction: &Transaction, disputes: &mut HashMap<u32, Transaction>) {
    // if there is a dispute corresponding to this transaction, fill missing data
    match transaction.r#type {
      TransactionType::Deposit | TransactionType::Withdrawal => {
        match disputes.get_mut(&transaction.tx) {
          Some(dispute) => {
            dispute.r#type = transaction.r#type;
            dispute.amount = transaction.amount;
          },
          _ => {}
        }
      },
      _ => {}
    }
}

fn commit_transaction(account: &mut Account, transaction: &Transaction, disputes: &HashMap<u32, Transaction>) {
    match transaction.r#type {
      TransactionType::Deposit => {
        account.deposit(transaction.amount.unwrap())
      }
      TransactionType::Withdrawal => {
        account.withdraw(transaction.amount.unwrap())
      }
      TransactionType::Dispute => {
        match find_dispute_for_transaction(&transaction, disputes) {
          Some(d) => {
            account.dispute(d.amount.unwrap())
          }
          None => {}
        }
      }
      TransactionType::Resolve => {
        match find_dispute_for_transaction(&transaction, disputes) {
          Some(d) => {
            account.resolve(d.amount.unwrap())
          }
          None => {}
        }
      }
      TransactionType::Chargeback => {
        match find_dispute_for_transaction(&transaction, disputes) {
          Some(d) => {
            account.chargeback(d.amount.unwrap())
          }
          None => {}
        }
      }
      _ => { panic!("Transaction type not implemented: {:?}", transaction.r#type ) }
    }
}

fn find_dispute_for_transaction<'a>(transaction: &'a Transaction, disputes: &'a HashMap<u32, Transaction>) -> Option<&'a Transaction> {
  let dispute = disputes.get(&transaction.tx)?;
  match dispute.r#type {
    TransactionType::Unknown => { None }
    _ => { Some(dispute) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use csv::{Trim,ReaderBuilder};
  use rust_decimal_macros::dec;

  fn reader(data: &str) -> Reader<io::Cursor<&str>> {
    let mut reader = ReaderBuilder::new();
    reader.has_headers(true).trim(Trim::All).from_reader(io::Cursor::new(data))
  }

  #[test]
  fn deposit() -> Result<(), Error> {
    let csv = "type, client, tx, amount
                deposit, 1, 1, 1.0
                deposit, 2, 3, 1.0
                deposit, 1, 2, 1.3";
    let result = process(&mut reader(csv))?;
    let account1 = result.get(&1).unwrap();
    let account2 = result.get(&2).unwrap();
    assert_eq!(account1.available, dec!(2.3));
    assert_eq!(account2.available, dec!(1.0));
    Ok(())
  }

  #[test]
  fn withdrawal() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 1.0
              deposit, 2, 4, 1.0
              withdrawal, 1, 1, 1.3";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(0.7));
    Ok(())
  }

  #[test]
  fn withdrawal_not_enough_credit() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 1.0
              withdrawal, 1, 1, 3.0";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(2.0));
    Ok(())
  }

  #[test]
  fn withdrawal_different_accounts() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 2, 2, 1.0
              withdrawal, 2, 1, 0.5";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(1.0));
    Ok(())
  }

  #[test]
  fn single_dispute() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 2.0
              dispute, 1, 1,";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(2.0));
    assert_eq!(account.held, dec!(1.0));
    Ok(())
  }

  #[test]
  fn dispute_unknown_transaction() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 2.0
              dispute, 1, 3,";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(3.0));
    assert_eq!(account.held, dec!(0.0));
    Ok(())
  }

  #[test]
  fn dispute_resolve() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 2.0
              dispute, 1, 1,
              resolve, 1, 1,";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(3.0));
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.locked, false);
    Ok(())
  }

  #[test]
  fn dispute_resolve_unknown() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 2.0
              dispute, 1, 1,
              resolve, 1, 3,";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(2.0));
    assert_eq!(account.held, dec!(1.0));
    assert_eq!(account.locked, false);
    Ok(())
  }

  #[test]
  fn dispute_chargeback() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 2.0
              dispute, 1, 1,
              chargeback, 1, 1,";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(2.0));
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.locked, true);
    Ok(())
  }

  #[test]
  fn operations_on_locked_account() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 2.0
              dispute, 1, 1,
              chargeback, 1, 1,
              deposit, 1, 3, 1.0
              withdrawal, 1, 4, 1.0
              dispute, 1, 3,
              chargeback, 1, 4,";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(2.0));
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.locked, true);
    Ok(())
  }

  #[test]
  fn dispute_chargeback_unknown() -> Result<(), Error> {
    let csv = "type, client, tx, amount
              deposit, 1, 1, 1.0
              deposit, 1, 2, 2.0
              dispute, 1, 1,
              chargeback, 1, 3,";
    let result = process(&mut reader(csv))?;
    let account = result.get(&1).unwrap();
    assert_eq!(account.available, dec!(2.0));
    assert_eq!(account.held, dec!(1.0));
    assert_eq!(account.locked, false);
    Ok(())
  }
}
