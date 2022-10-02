use std::str::FromStr;

use serde::{Serialize, Deserialize, Deserializer, de::Error};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use super::tx_engine::transaction::{TxType, Transaction};


#[derive(Debug, Clone, Deserialize)]
pub struct InputColumns {
    #[serde(rename = "type")]
    pub transaction_type: String,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    #[serde(rename = "amount", with = "rust_decimal::serde::str_option")]
    pub amount: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputColumns {
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "available", with = "rust_decimal::serde::str")]
    pub available_amount: Decimal,
    #[serde(rename = "held", with = "rust_decimal::serde::str")]
    pub held_amount: Decimal,
    #[serde(rename = "total", with = "rust_decimal::serde::str")]
    pub total: Decimal,
    #[serde(rename = "locked")]
    pub locked: bool,
}

impl From<InputColumns> for Transaction {
    fn from(input: InputColumns) -> Self {
        Transaction::new(
            input.transaction_id,
            // !PANICS: following conversion can cause panic if the transaction
            // type is not known
            TxType::from_str(&input.transaction_type).expect("Malformed Data"),
            // Set to None if conversion to deciaml fails
            input.amount
        )
    }
}

mod tests {
    use serde_test::assert_de_tokens;
    use super::InputColumns;

    #[test]
    fn test_limits() {
        //let de_data = InputColumns { transaction_type: "deposit".to_owned() , client_id: 5, transaction_id: 9, amount: Some(1.7976931348623157e+308)};
        dbg!(de_data);
    }

}
