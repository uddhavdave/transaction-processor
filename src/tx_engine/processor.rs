use anyhow::Result as Result;
use super::{transaction::Transaction, account::Statement};

pub type ClientId = u16;

/// ['AccountStore'] defines the behaviour of the interface which is used for
/// storing account data and modifying it.
/// It is generic over structures which implement [`Into<Transaction>`], needed
/// for adding the data in client's account.
/// Users are expected to receive data in [`Statement`]
pub trait AccountStore <I: Into<Transaction>> {
    fn add_transaction_to_client_account(&mut self, client_id: ClientId, input_data: I) -> Result<()>;
    fn get_client_account_statement(&self, client_id: ClientId) -> Result<Statement>;
    fn get_clients_list(&self) -> Result<Vec<ClientId>>;
}
