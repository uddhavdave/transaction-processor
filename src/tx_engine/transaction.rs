use anyhow::Result;
use std::str::FromStr;
use super::errors::TxEngineErrors;
use rust_decimal::Decimal;

pub type TxId = u32;
pub type Amount = Decimal;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TxType {
    Deposit,
    Withdraw,
    Dispute,
    Resolve,
    Chargeback,
}

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Transaction {
    pub tx_id: TxId,
    pub tx_type: TxType,
    pub amount: Option<Amount>,
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
