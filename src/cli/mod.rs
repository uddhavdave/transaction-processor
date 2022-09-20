use clap::Parser;
use std::path::PathBuf;

/// This struct parses the arguments passed to program.
/// We expect the user to provide a filename, otherwise it fails
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Mandatory File path 
    #[clap(value_parser = validate_file, value_name = "CSV_FILE")]
    file_path : PathBuf,
}

/// Following function is a wrapper around `parse` method from derive as its not
pub fn get_args() -> PathBuf {
    // Program will error out if the arguments supplied are not upto mark
    Cli::parse().file_path
}

/// Following function validates input is CSV file
fn validate_file(s: &str) -> Result<PathBuf, String> {
    if s.ends_with(".csv") {
        Ok(PathBuf::from(s))
    } else {
        Err(format!(
            "File Provided is not csv"
        ))
    }
}
