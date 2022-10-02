use thiserror::Error;
use tokio::sync::watch::error;

#[derive(Error, Debug)]
pub enum TxEngineErrors {
    /// Following Error is generally raised in Withdraw Transaction
    /// or Dispute Transaction
    #[error("Insufficient funds in account")]
    InsufficientFunds,
    /// Following error is returned if a Dispute transaction is raised without 
    /// known Transaction ID
    #[error("Transaction ID {0} doesn't exist")]
    TxDoesNotExist(u32),
    #[error("Transaction ID {0} already disputed")]
    TxAlreadyDisputed(u32),
    /// Following error is returned if a Transaction already exists in History
    /// and there is an attempt to change it
    #[error("Transaction ID {0} already exists")]
    TxExists(u32),
    #[error("Transaction ID {0} not disputed")]
    TxNotDisputed(u32),
    /// Invalid Input can be returned if the Transaction Type has missing fields
    /// For Ex, If Deposit Transaction has no amount attached to it.
    /// Since `Amount` is wrapped in Option, it can be set to `None`
    /// In such case, the program would simply fail the trasaction and continue
    #[error("Invalid Input")]
    InvalidInput,
    /// Assumption: Account should be locked if a Dispute claim is made on 
    /// Withdraw Transaction, since the money is already psiphoned out of the
    /// account.
    #[error("Account locked due to chargeback/dispute on withdraw")]
    ClientAccountLocked,
    #[error("Withdraw Transaction cannot be disputed")]
    WithdrawDisputeError,
    #[error("unknown error")]
    Unknown,
}
