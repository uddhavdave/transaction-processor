mod csv_utils;
mod cli;
mod tx_engine;
mod client_store;
mod io_types;

use std::io;

use client_store::ClientCache;
use anyhow::Result as Result;
use io_types::{ InputColumns, OutputColumns };
use tx_engine::processor::AccountStore;

/// Currently, the processor blocks the execution which is not efficient
/// [`AccountStore`] trait should be async inorder to increase efficiency
/// This will allow multiple threads to work on same Account which is guarded by
/// [`RwLock`]
fn main() -> Result<()> {
    let mut file_reader = csv_utils::csv_reader::create_reader(cli::get_args())?;

    let mut client_store = ClientCache::new();

    for raw_data in file_reader.deserialize() {
        let tx : InputColumns = raw_data?;

        if let Err(err) = client_store.add_transaction_to_client_account(tx.client_id, tx.into()) {
            eprintln!("Processor Error: {}", err);
        }
    }

    // Fetch all accounts statements and write to stdout as CSV
    let mut writer = csv::Writer::from_writer(io::stdout());

    for client in client_store.get_clients_list()? {
        let statement = client_store.get_client_account_statement(client)?;
        writer.serialize(OutputColumns {
            client_id: client,
            available_amount: statement.available,
            held_amount: statement.held,
            total: statement.total,
            locked: statement.locked,
        })?;
    }

    Ok(())
}
