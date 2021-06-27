use serde::Deserialize;
use serde::Serialize;

#[derive(Debug,Serialize,Deserialize,PartialEq,Copy,Clone)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
  Unknown,
  Deposit,
  Withdrawal,
  Dispute,
  Resolve,
  Chargeback
}
