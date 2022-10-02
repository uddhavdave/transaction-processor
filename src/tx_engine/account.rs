use anyhow::Result as Result;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use super::errors::TxEngineErrors;
use super::transaction::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Statement {
    pub total: Amount,
    pub available: Amount, 
    pub held: Amount,
    pub locked: bool,
}

impl Statement {
    pub fn new(total: Amount, available: Amount, held: Amount, locked: bool) -> Self {
        Statement { total, available, held, locked }
    }
}

#[derive(Default, Debug)]
pub struct Account {
    pub total: Amount,
    pub held: Amount,
    pub available: Amount,
    pub history: HashMap<TxId, Transaction>,
    pub locked: bool,
}

impl Account {
    pub fn get_statement(&self) -> Result<Statement> {
        Ok(
            Statement {
                total: self.total.round_dp(4),
                available: self.available.round_dp(4),
                locked: self.locked,
                held: self.held.round_dp(4),
            }
        )
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn new() -> Self {
        Account { locked: false, total: Decimal::new(0, 4), held: Decimal::new(0, 4), available: Decimal::new(0, 4), history: HashMap::new() }
    }

    pub fn add_transaction(&mut self, tx: Transaction) ->  Result<()> {
        if self.is_locked() {
            return Err(TxEngineErrors::ClientAccountLocked.into());
        }

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

    fn handle_dispute(&mut self, disputed_tx_id: TxId) -> Result<()> {
        match self.history.entry(disputed_tx_id) {
            Entry::Occupied(mut entry) => {
                let disputed_tx = entry.get_mut();

                if disputed_tx.is_disputed() {
                    return Err(TxEngineErrors::TxAlreadyDisputed(disputed_tx_id).into())
                }

                if disputed_tx.tx_type == TxType::Withdraw {
                    // ASSUMPTION:
                    // Currently we assume that a Withdraw cannot be disputed,
                    // as handling the transaction is outside of the scope of
                    // program. However, we should lock the account in this case
                    self.locked = true;
                    return Err(TxEngineErrors::WithdrawDisputeError.into())
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
                self.locked = true;

                disputed_tx.tx_type = TxType::Chargeback;
            }
            Entry::Vacant(_) => {
                return Err(TxEngineErrors::TxDoesNotExist(disputed_tx_id).into())
            }
        };
        Ok(())
    }
}

macro_rules! add_transaction {
    ($acc:tt, $txid:tt, $txtype:ident, $($amt:tt)?) => {
        $acc.add_transaction(Transaction::new($txid, TxType::$txtype, Some(dec!($($amt)?)))).unwrap();
    };
    ($acc:tt, $txid:tt, $txtype:ident) => {
        $acc.add_transaction(Transaction::new($txid, TxType::$txtype, None)).unwrap();
    }
}

mod tests {
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal_macros::dec;
    use crate::tx_engine::account;

    use super::*;

    #[test]
    fn test_dispute() -> Result<()>{
        let mut account = Account::new();
        add_transaction!(account, 1, Deposit, 10.0);
        assert_eq!(account.get_statement()?, Statement::new(dec!(10.0), dec!(10.0), dec!(0), false ));
        add_transaction!(account, 1, Dispute);
        assert_eq!(account.get_statement()?, Statement::new(dec!(10.0), dec!(0.0), dec!(10.0), false ));
        Ok(())
    }

    #[test]
    fn test_chargeback() -> Result<()> {
        let mut account = Account::new();
        add_transaction!(account, 1, Deposit, 10.0);
        add_transaction!(account, 1, Dispute);
        add_transaction!(account, 1, Chargeback);
        assert_eq!(account.get_statement()?, Statement::new(dec!(0), dec!(0), dec!(0), true));
        Ok(())
    }

    #[test]
    fn test_resolve() -> Result<()> {
        let mut account = Account::new();
        add_transaction!(account, 1, Deposit, 10.0);
        add_transaction!(account, 1, Dispute);
        add_transaction!(account, 1, Resolve);
        assert_eq!(account.get_statement()?, Statement::new(dec!(10), dec!(10), dec!(0), false));
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_insufficient_funds() {
        let mut acc = Account::new();
        add_transaction!(acc, 1, Deposit, 4);
        // Should panic below because of wrap call
        add_transaction!(acc, 2, Withdraw, 10);

        // If transaction was added then check balance is not less than 0
        assert_eq!(acc.get_statement().unwrap(), Statement::new(dec!(4), dec!(4), dec!(0), false));
    }

    #[test]
    #[should_panic]
    fn test_withdraw_dispute() {
        let mut acc = Account::new();
        add_transaction!(acc, 1, Deposit, 5);
        add_transaction!(acc, 3, Withdraw, 5);
        // Should panic below because Withdraw transaction cannot be disputed
        add_transaction!(acc, 3, Dispute);
    }
}
