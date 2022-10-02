use std::{collections::HashMap, sync::{Arc, RwLock, PoisonError}};
use anyhow::Result;

use crate::tx_engine::processor::{
    AccountStore, ClientId
};
use crate::tx_engine::account::{Account, Statement};
use crate::tx_engine::errors::TxEngineErrors;
use super::io_types::InputColumns;

/// Cache stores the Transactions against ClientId
/// Simple implementation of an [`AccountStore`]
/// ClientId is used as key here to reduce collisions, when compared to Transactions
/// Value against the `ClientId` is the State of the account which is [`Open`] by default
pub struct ClientCache {
    pub store: HashMap<ClientId, Arc<RwLock<Account>>>
}

impl ClientCache {
    pub fn new() -> Self {
        ClientCache { store: HashMap::new() }
    }
}

fn rwlock_error<T>(_ : PoisonError<T>) -> TxEngineErrors {
    TxEngineErrors::Unknown
}

impl AccountStore<InputColumns> for ClientCache {
    fn add_transaction_to_client_account(&mut self, client_id: ClientId, input_data: InputColumns) -> Result<()> {
        self.store
            .entry(client_id)
            .or_insert_with(|| {
                Arc::new(RwLock::new(Account::new()))
            })
            .write()
            .map_err(rwlock_error)?
            .add_transaction(input_data.into())
    }

    fn get_client_account_statement(&self, client_id: ClientId) -> Result<Statement> {
        self.store
            .get(&client_id)
            .ok_or(TxEngineErrors::InvalidInput)?
            .read()
            .map_err(rwlock_error)?
            .get_statement()
    }

    fn get_clients_list(&self) -> Result<Vec<ClientId>> {
        Ok(self.store.keys().map(|key| *key).collect::<Vec<ClientId>>())
    }
}
