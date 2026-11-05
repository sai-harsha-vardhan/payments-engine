use super::PaymentsEngine;
use crate::types::account::{Account, ClientId};
use crate::types::errors::DomainError;
use crate::types::transaction::{
    TransactionId, TransactionKind, TransactionRecord, TransactionStatus,
};

impl PaymentsEngine {
    pub fn dispute(
        &mut self,
        client: ClientId,
        transaction_id: TransactionId,
    ) -> Result<(), DomainError> {
        let transaction_record =
            self.get_transaction_record_for_a_client(client, transaction_id)?;
        if transaction_record.kind != TransactionKind::Deposit
            || transaction_record.status != TransactionStatus::Settled
        {
            return Err(DomainError::NotDisputable);
        }
        let account = self.get_account(client)?;

        let updated = Account {
            available: account
                .available
                .checked_sub(transaction_record.amount)
                .ok_or(DomainError::Overflow)?,
            held: account
                .held
                .checked_add(transaction_record.amount)
                .ok_or(DomainError::Overflow)?,
            ..account
        };

        self.update_records(client, updated, transaction_id, TransactionStatus::Disputed)
    }

    pub fn resolve(
        &mut self,
        client: ClientId,
        transaction_id: TransactionId,
    ) -> Result<(), DomainError> {
        let transaction_record = self.disputed_deposit(client, transaction_id)?;
        let account = self.get_account(client)?;

        let updated = Account {
            available: account
                .available
                .checked_add(transaction_record.amount)
                .ok_or(DomainError::Overflow)?,
            held: account
                .held
                .checked_sub(transaction_record.amount)
                .ok_or(DomainError::Overflow)?,
            ..account
        };
        // Transaction state goes back to settled after a resolve, assuming a second dispute can occur and our state machine should allow it
        self.update_records(client, updated, transaction_id, TransactionStatus::Settled)
    }

    pub fn chargeback(
        &mut self,
        client: ClientId,
        transaction_id: TransactionId,
    ) -> Result<(), DomainError> {
        let transaction_record = self.disputed_deposit(client, transaction_id)?;
        let account = self.get_account(client)?;

        let updated = Account {
            held: account
                .held
                .checked_sub(transaction_record.amount)
                .ok_or(DomainError::Overflow)?,
            locked: true,
            ..account
        };

        self.update_records(
            client,
            updated,
            transaction_id,
            TransactionStatus::ChargedBack,
        )
    }

    fn get_account(&self, client: ClientId) -> Result<Account, DomainError> {
        self.accounts
            .get(client)
            .ok_or(DomainError::UnknownTransactionId)
    }

    /// The record for `transaction_id`, which must belong to `client`
    fn get_transaction_record_for_a_client(
        &self,
        client: ClientId,
        transaction_id: TransactionId,
    ) -> Result<TransactionRecord, DomainError> {
        let transaction_record = self.transactions.get_transaction_record(transaction_id)?;
        if transaction_record.client != client {
            return Err(DomainError::ClientMismatch);
        }
        Ok(transaction_record)
    }

    /// Resolve and chargeback only apply to a deposit that is currently disputed
    fn disputed_deposit(
        &self,
        client: ClientId,
        transaction_id: TransactionId,
    ) -> Result<TransactionRecord, DomainError> {
        let transaction_record =
            self.get_transaction_record_for_a_client(client, transaction_id)?;
        if transaction_record.status != TransactionStatus::Disputed {
            return Err(DomainError::NotUnderDispute);
        }
        Ok(transaction_record)
    }
}
