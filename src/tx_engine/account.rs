use anyhow::Result;
use std::collections::HashMap;
use std::str::FromStr;
use std::collections::hash_map::Entry;
use super::errors::TxEngineErrors;
use rust_decimal::Decimal;
use super::io_types::{OutputColumns, InputColumns};

pub type ClientId = u16;

pub type TxId = u32;
pub type Amount = Decimal;
pub type Statement = (Amount, Amount, Amount, bool);

#[derive(Debug, PartialEq)]
pub enum TxType {
    Deposit,
    Withdraw,
    Dispute,
    Resolve,
    Chargeback,
}

struct Dispute(TxId);
struct Resolve(TxId);
struct Chargeback(TxId);

impl FromStr for TxType {
    type Err = TxEngineErrors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deposit" => Ok(TxType::Deposit),
            "withdrawal" => Ok(TxType::Withdraw),
            "dispute" => Ok(TxType::Dispute),
            "resolve" => Ok(TxType::Resolve),
            "chargeback" => Ok(TxType::Chargeback),
            _ => Err(TxEngineErrors::Unknown)
        }
    }
}

#[derive(Debug)]
pub struct Transaction {
    pub tx_id: TxId,
    pub tx_type: TxType,
    pub amount: Option<Amount>,
}

impl From<InputColumns> for Transaction {
    fn from(input: InputColumns) -> Self {
        Transaction {
            tx_id: input.transaction_id,
            // !PANICS: following conversion can cause panic if the transaction
            // type is not known
            tx_type: TxType::from_str(&input.transaction_type).expect("Malformed Data"),
            amount: input.amount
        }
    }
}

impl Transaction {
    pub fn new(tx_id: TxId, tx_type: TxType, amount: Option<Amount>) -> Self {
        Transaction {
            tx_id,
            tx_type,
            amount
        }
    }

    pub fn is_disputed(&self) -> bool {
        if self.tx_type == TxType::Dispute {
            true
        } else {
            false
        }
    }
}

#[derive(Default, Debug)]
pub struct AccountState {
    locked: bool,
    total: Amount,
    held: Amount,
    available: Amount,
    history: HashMap<TxId, Transaction>
}

impl AccountState {
    pub fn new() -> Self {
        AccountState::default()
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn add_transaction(&mut self, tx: Transaction) ->  Result<()> {
        match tx.tx_type {
            TxType::Deposit => {
                if let Some(amount) = tx.amount {
                    self.total += amount;
                    self.available += amount;
                } else {
                    return Err(TxEngineErrors::InvalidInput.into());
                }

                // Add trasaction to client's history
                match self.history.entry(tx.tx_id) {
                    // If TX already present, consider this an error and ignore
                    Entry::Occupied(_) => return Err(TxEngineErrors::TxExists(tx.tx_id).into()),
                    // Insert transaction if ID not present
                    Entry::Vacant(entry) => entry.insert(tx)
                };
            }
            TxType::Withdraw => {
                if let Some(amount) = tx.amount {
                    if self.total < amount {
                        return Err(TxEngineErrors::InsufficientFunds.into());
                    }

                    // Withdraw funds
                    self.total -= amount;
                    self.available -= amount;
                } else {
                    return Err(TxEngineErrors::InvalidInput.into());
                }

                // Add trasaction to client's history
                match self.history.entry(tx.tx_id) {
                    // If TX already present, consider this an error and ignore
                    Entry::Occupied(_) => return Err(TxEngineErrors::TxExists(tx.tx_id).into()),
                    // Insert transaction if ID not present
                    Entry::Vacant(entry) => entry.insert(tx)
                };
            }
            TxType::Dispute => {
                self.handle_dispute(tx.tx_id)?;
            }
            TxType::Resolve => {
                self.handle_resolve(tx.tx_id)?;
            }
            TxType::Chargeback => {
                self.handle_chargeback(tx.tx_id)?;
            }
        };
        Ok(())
    }

    pub fn get_statement(&self) -> Statement {
        (self.total, self.held, self.available, self.locked)
    }

    fn handle_dispute(&mut self, disputed_tx_id: TxId) -> Result<()> {
        match self.history.entry(disputed_tx_id) {
            Entry::Occupied(mut entry) => {
                let disputed_tx = entry.get_mut();

                if disputed_tx.is_disputed() {
                    return Err(TxEngineErrors::TxAlreadyDisputed(disputed_tx_id).into())
                }

                // SAFE UNWRAP: Transaction amount ensured while
                // inserting entry in history
                let disputed_amount = disputed_tx.amount.unwrap();

                // Shift amount to held state
                // Check if available amount has more that the disputed
                // amount
                if self.available >= disputed_amount {
                    self.available -= disputed_amount;
                    self.held += disputed_amount;

                    // Change tx_type to disputed
                    // This will be needed for resolve/chargeback
                    disputed_tx.tx_type = TxType::Dispute;
                } else {
                    return Err(TxEngineErrors::InsufficientFunds.into())
                }
            }
            Entry::Vacant(_) => {
                return Err(TxEngineErrors::TxDoesNotExist(disputed_tx_id).into())
            }
        };
        Ok(())
    }

    fn handle_resolve(&mut self, disputed_tx_id: TxId) -> Result<()> {
        match self.history.entry(disputed_tx_id) {
            Entry::Occupied(mut entry) => {
                let disputed_tx = entry.get_mut();

                if !disputed_tx.is_disputed() {
                    return Err(TxEngineErrors::TxNotDisputed(disputed_tx_id).into())
                }

                let disputed_amount = disputed_tx.amount.unwrap();

                // There is not scenario where held amount is less than disputed
                self.held -= disputed_amount;
                self.available += disputed_amount;

                disputed_tx.tx_type = TxType::Resolve;
            }
            Entry::Vacant(_) => {
                return Err(TxEngineErrors::TxDoesNotExist(disputed_tx_id).into())
            }
        };
        Ok(())
    }

    fn handle_chargeback(&mut self, disputed_tx_id: TxId) -> Result<()> {
        match self.history.entry(disputed_tx_id) {
            Entry::Occupied(mut entry) => {
                let disputed_tx = entry.get_mut();

                if !disputed_tx.is_disputed() {
                    return Err(TxEngineErrors::TxNotDisputed(disputed_tx_id).into())
                }

                let disputed_amount = disputed_tx.amount.unwrap();

                self.held -= disputed_amount;
                self.total -= disputed_amount;

                disputed_tx.tx_type = TxType::Chargeback;

                self.locked = true;
            }
            Entry::Vacant(_) => {
                return Err(TxEngineErrors::TxDoesNotExist(disputed_tx_id).into())
            }
        };
        Ok(())
    }
}
