use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::types::errors::DomainError;
use crate::types::transaction::{TransactionId, TransactionRecord, TransactionStatus};

#[derive(Default)]
pub struct TransactionStore(HashMap<TransactionId, TransactionRecord>);

impl TransactionStore {
    pub fn insert_new(
        &mut self,
        transaction_id: TransactionId,
        transaction_record: TransactionRecord,
    ) -> Result<(), DomainError> {
        match self.0.entry(transaction_id) {
            Entry::Occupied(_) => Err(DomainError::DuplicateTransaction),
            Entry::Vacant(slot) => {
                slot.insert(transaction_record);
                Ok(())
            }
        }
    }

    pub fn get_transaction_record(
        &self,
        transaction_id: TransactionId,
    ) -> Result<TransactionRecord, DomainError> {
        self.0
            .get(&transaction_id)
            .copied()
            .ok_or(DomainError::UnknownTransactionId)
    }

    pub fn set_status(
        &mut self,
        transaction_id: TransactionId,
        status: TransactionStatus,
    ) -> Result<(), DomainError> {
        let transaction_record = self
            .0
            .get_mut(&transaction_id)
            .ok_or(DomainError::UnknownTransactionId)?;
        transaction_record.status = status;
        Ok(())
    }
}
