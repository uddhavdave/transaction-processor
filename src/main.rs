mod csv_utils;
mod cli;
mod tx_engine;
mod errors;

use transaction_engine::process;
use anyhow::Result;

fn main() -> Result<()> {
    let file_reader = csv_utils::csv_reader::create_reader(cli::get_args())?;

    let _ = process(file_reader)?;

    Ok(())
}
