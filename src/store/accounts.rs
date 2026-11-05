use std::collections::HashMap;

use crate::types::account::{Account, ClientId};

#[derive(Default)]
pub struct AccountStore(HashMap<ClientId, Account>);

impl AccountStore {
    pub fn get(&self, client: ClientId) -> Option<Account> {
        self.0.get(&client).copied()
    }

    /// The client's account, created with zero balances if it does not exist yet.
    pub fn get_or_create(&mut self, client: ClientId) -> Account {
        *self.0.entry(client).or_default()
    }

    pub fn set(&mut self, client: ClientId, account: Account) {
        self.0.insert(client, account);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ClientId, &Account)> {
        self.0.iter()
    }
}
