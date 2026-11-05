mod deposit;
mod dispute;
mod withdrawal;

use crate::types::Command;
use crate::types::account::{Account, ClientId};
use crate::types::errors::DomainError;
use crate::types::transaction::{TransactionId, TransactionRecord, TransactionStatus};

use crate::store::accounts::AccountStore;
use crate::store::transactions::TransactionStore;

#[derive(Default)]
pub struct PaymentsEngine {
    accounts: AccountStore,
    transactions: TransactionStore,
}

impl PaymentsEngine {
    pub fn new() -> Self {
        PaymentsEngine::default()
    }

    pub fn apply(&mut self, command: Command) -> Result<(), DomainError> {
        match command {
            Command::Deposit {
                client,
                transaction_id,
                amount,
            } => self.deposit(client, transaction_id, amount),
            Command::Withdrawal {
                client,
                transaction_id,
                amount,
            } => self.withdraw(client, transaction_id, amount),
            Command::Dispute {
                client,
                transaction_id,
            } => self.dispute(client, transaction_id),
            Command::Resolve {
                client,
                transaction_id,
            } => self.resolve(client, transaction_id),
            Command::Chargeback {
                client,
                transaction_id,
            } => self.chargeback(client, transaction_id),
        }
    }

    pub fn accounts(&self) -> impl Iterator<Item = (&ClientId, &Account)> {
        self.accounts.iter()
    }

    /// Creates the transaction record, and returns the client's account created if new
    fn create_records(
        &mut self,
        transaction_id: TransactionId,
        transaction_record: TransactionRecord,
    ) -> Result<Account, DomainError> {
        self.transactions
            .insert_new(transaction_id, transaction_record)?;
        Ok(self.accounts.get_or_create(transaction_record.client))
    }

    /// Writes the new status of the transaction and the new updated account details
    /// This is the commit step of every flow
    fn update_records(
        &mut self,
        client: ClientId,
        account: Account,
        transaction_id: TransactionId,
        status: TransactionStatus,
    ) -> Result<(), DomainError> {
        self.transactions.set_status(transaction_id, status)?;
        self.accounts.set(client, account);
        Ok(())
    }
}
