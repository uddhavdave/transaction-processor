use anyhow::{
    Result,
    bail
};
use std::path::PathBuf;
use std::fs::File;

/// Buffered Reader for CSV file
pub mod csv_reader {
    use csv::{Reader, ReaderBuilder, Trim};
    use super::*;

    pub fn create_reader(path: PathBuf) -> Result<Reader<File>> {
        // Check whether path is correct
        if !path.exists() {
            // SAFE UNWRAP: UTF-8 validity checked in CLI input
            bail!("Path {} does not exists", path.as_path().to_str().expect("UTF-8 valid string"));
        }

        let reader = ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_path(path.as_path())?;
        Ok(reader)
    }
}

