mod tx_engine;

use std::io;
use tx_engine::io_types::*;
use anyhow::Result;
//use futures::io::AsyncRead;
use std::io::Read;
use tx_engine::processor::ClientCache;

/// Function is guarded by TraitBounds
/// Any Type that implements `Read`(usually File) can be wrapped in `csv::Reader` and passed 
/// to this Function
/// TODO: Make it async and use `AllowStdIo`/`AsyncRead`
pub fn process<T: Read>(mut reader: csv::Reader<T>) -> Result<()> {
    let mut records = ClientCache::new();
    for raw_data in reader.deserialize() {
        // Any errors is reading the CSV are propagated futher
        let tx : InputColumns = raw_data?;
        dbg!(&tx);

        // We can store errors given out by processing function to debug
        if let Err(err) = records.process_raw_transaction(tx.client_id, tx.into()) {
            eprintln!("Processor Error: {}", err);
        }
    }

    let mut writer = csv::Writer::from_writer(io::stdout());
    records.drain_account_statements().collect::<Vec<OutputColumns>>()
        .into_iter()
        .try_for_each(|statement| writer.serialize(statement))?;
    Ok(())
}
