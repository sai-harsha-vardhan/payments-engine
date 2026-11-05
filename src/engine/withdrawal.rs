use super::PaymentsEngine;
use crate::types::account::{Account, ClientId};
use crate::types::amount::Amount;
use crate::types::errors::DomainError;
use crate::types::transaction::{
    TransactionId, TransactionKind, TransactionRecord, TransactionStatus,
};

impl PaymentsEngine {
    pub fn withdraw(
        &mut self,
        client: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    ) -> Result<(), DomainError> {
        if !amount.is_positive() {
            return Err(DomainError::InvalidAmount);
        }

        let transaction_record = TransactionRecord {
            client,
            amount,
            kind: TransactionKind::Withdrawal,
            status: TransactionStatus::Created,
        };
        let account = self.create_records(transaction_id, transaction_record)?;

        account.ensure_unlocked()?;
        if account.available < amount {
            return Err(DomainError::InsufficientBalance);
        }

        let updated = Account {
            available: account
                .available
                .checked_sub(amount)
                .ok_or(DomainError::Overflow)?,
            ..account
        };

        self.update_records(client, updated, transaction_id, TransactionStatus::Settled)
    }
}
