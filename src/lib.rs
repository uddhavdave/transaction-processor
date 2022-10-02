mod tx_engine;
use serde::de::DeserializeOwned;
//use std::sync::Arc;
use tx_engine::processor::ClientId;
use tx_engine::transaction::Transaction;

/// Trait defines Input types to transaction engine
pub trait InputType: Into<Transaction> + DeserializeOwned {
    /// We expect the InputType to have client id mandatorily, since otherwise
    /// no processing can be performed
    fn get_client_id(&self) -> ClientId;
}
