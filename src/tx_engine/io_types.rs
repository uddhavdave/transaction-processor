use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Deserialize)]
pub struct InputColumns {
    #[serde(rename = "type")]
    pub transaction_type: String,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub amount: Option<Decimal>,
}


#[derive(Debug, Serialize)]
pub struct OutputColumns {
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "available", with = "rust_decimal::serde::float")]
    pub available_amount: Decimal,
    #[serde(rename = "held", with = "rust_decimal::serde::float")]
    pub held_amount: Decimal,
    #[serde(rename = "total", with = "rust_decimal::serde::float")]
    pub total: Decimal,
    #[serde(rename = "locked")]
    pub locked: bool,
}
