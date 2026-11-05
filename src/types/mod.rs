pub(crate) mod account;
pub(crate) mod amount;
pub mod errors;
pub(crate) mod transaction;

use account::ClientId;
use amount::Amount;
use transaction::TransactionId;

pub enum Command {
    Deposit {
        client: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    },
    Withdrawal {
        client: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    },
    Dispute {
        client: ClientId,
        transaction_id: TransactionId,
    },
    Resolve {
        client: ClientId,
        transaction_id: TransactionId,
    },
    Chargeback {
        client: ClientId,
        transaction_id: TransactionId,
    },
}
