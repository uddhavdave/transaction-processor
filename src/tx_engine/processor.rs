use super::account::*;
use super::errors::TxEngineErrors;
use super::io_types::OutputColumns;
use std::collections::HashMap;
use anyhow::Result;

/// Cache stores the Transactions against ClientId
/// ClientId is used as key here since its `u16`
///
/// Value against the `ClientId` is the State of the account
pub struct ClientCache {
    pub store: HashMap<ClientId, AccountState>
}

impl ClientCache {
    pub fn new() -> Self {
        ClientCache { store: HashMap::new() }
    }

    /// This function processes a single transaction for an account
    /// It checks for the account status (locked/Unlocked)
    /// Assuming Locked accounts, cannot be operated on it skips processing the transaction
    pub fn process_raw_transaction(&mut self, client: ClientId, tx: Transaction) -> Result<()> {
        // If the client is not present then create one and proceed with adding
        // Transaction
        let client_account = self.store
            .entry(client)
            .or_insert_with(|| AccountState::new());
        if client_account.is_locked() {
            return Err(TxEngineErrors::ClientAccountLocked.into());
        }

        client_account.add_transaction(tx)
    }

    /// This function returns an owned iterator, which consumes `store`
    /// 
    /// All `AccountState` initialize with Default values
    /// In case, the Client Cache is populated, Iterator returns `None`
    pub fn drain_account_statements(self) -> impl Iterator<Item=OutputColumns> {
        self.store
            .into_iter()
            .map( |(client, account)| {
                    let (total, held, available, locked) = account.get_statement();
                    OutputColumns {
                        client_id: client,
                        available_amount: available,
                        held_amount: held,
                        total,
                        locked
                    }
                })
    }
}
