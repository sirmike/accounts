use serde::Deserialize;

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
  Deposit,
  Withdrawal,
  Dispute,
  Resolve,
  Chargeback
}
