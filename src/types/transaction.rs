use serde::Deserialize;

use super::account::ClientId;
use super::amount::Amount;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct TransactionId(pub u32);

#[derive(Clone, Copy, PartialEq)]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
}

/// Deposit: Created -> Settled -> Disputed -> Settled (resolve, may be disputed again) | ChargedBack (terminal).
/// Withdrawal: Created -> Settled.
#[derive(Clone, Copy, PartialEq)]
pub enum TransactionStatus {
    Created,
    Settled,
    Disputed,
    ChargedBack,
}

#[derive(Clone, Copy)]
pub struct TransactionRecord {
    pub client: ClientId,
    pub amount: Amount,
    pub kind: TransactionKind,
    pub status: TransactionStatus,
}
