use super::PaymentsEngine;
use crate::types::account::{Account, ClientId};
use crate::types::amount::Amount;
use crate::types::errors::DomainError;
use crate::types::transaction::{
    TransactionId, TransactionKind, TransactionRecord, TransactionStatus,
};

impl PaymentsEngine {
    pub fn deposit(
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
            kind: TransactionKind::Deposit,
            status: TransactionStatus::Created,
        };
        let account = self.create_records(transaction_id, transaction_record)?;

        account.ensure_unlocked()?;

        let updated = Account {
            available: account
                .available
                .checked_add(amount)
                .ok_or(DomainError::Overflow)?,
            ..account
        };
        updated.total().ok_or(DomainError::Overflow)?;

        self.update_records(client, updated, transaction_id, TransactionStatus::Settled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Command;

    const MAX: &str = "79228162514264337593543950335";

    #[test]
    fn deposit_that_overflows_the_total_is_rejected_and_changes_no_balance() {
        let client = ClientId(1);
        let deposit = |id: u32, amount: &str| Command::Deposit {
            client,
            transaction_id: TransactionId(id),
            amount: amount.parse().unwrap(),
        };
        let mut payments_engine = PaymentsEngine::default();
        payments_engine.apply(deposit(1, MAX)).unwrap();
        payments_engine
            .apply(Command::Dispute {
                client,
                transaction_id: TransactionId(1),
            })
            .unwrap();
        let balances = |engine: &PaymentsEngine| -> Vec<Account> {
            engine.accounts().map(|(_, account)| *account).collect()
        };
        let before = balances(&payments_engine);

        let result = payments_engine.apply(deposit(2, "1"));

        assert_eq!(result, Err(DomainError::Overflow));
        assert_eq!(balances(&payments_engine), before);
    }
}
