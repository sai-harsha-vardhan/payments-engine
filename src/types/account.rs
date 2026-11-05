use serde::Deserialize;

use super::amount::Amount;
use super::errors::DomainError;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub u16);

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Account {
    pub available: Amount,
    pub held: Amount,
    pub locked: bool,
}

impl Account {
    pub fn total(&self) -> Option<Amount> {
        self.available.checked_add(self.held)
    }

    pub fn ensure_unlocked(&self) -> Result<(), DomainError> {
        if self.locked {
            return Err(DomainError::AccountLocked);
        }
        Ok(())
    }
}
