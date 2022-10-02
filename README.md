# TRANSACTION ENGINE
## Introduction
This program provides functionality to process transaction history of client's 
asset account and output account details after processing all transactions.

Library exposes a `AccountStore` trait which can be used to implement the account storage to
keep track of Clients and Transaction history, giving it the flexibility to work
with any Cache implementation.

A cache is already implemented in [`client_store.rs`](./client_store.rs) which is
uses hashmap to store the Transactions per client. Transaction ID being a u32 type
is not suitable for a Key, hence optimal key choice is client ID which is u16,
ensuring far less collisions if tested against large dataset.

Errors and defined in [`errors.rs`](./tx_engine/errors.rs) which uses ThisError crate
to simplify the Describing errors. The Library also depends heavily in `anyhow` crate
to club different errors which implement `Error` trait. `clap` crate is used to 
parse the command line arguments.

Program expects the input csv to be deserilaized to `InputColumns` which is defined
in [`io_types.rs`] which also contains the output format of the program defined in
`OutputColumns`. Both these structs use serde attributes to guide de/serialization
of data. A custom attribute macro is used for decimal serializing.

All the calculations performed on Account Items are performed on `Decimal` objects
from `rust_decimal` instead of f64 since its not safe to use the latter for monetry
calculations.

## Usage
Program's CLI interface accepts csv filename as arguments and outputs the csv data on
STDOUT. The output can be piped to a csv file for storage purposes.
Following is a sample command:
`$ cargo run -- transaction.csv > accounts.csv`

## Notes
### Features
Basic Transaction types supported are:
- Deposit
- Withdraw
Special transaction types supported are:
- Dispute: Flags a transaction and holds the amount
- Resolve: Removes flag and deposits amount back to client's asset account
- Chargeback: Removes flag and credits amount to client's credit accounts

### Assumptions
- Withdraw transaction cannot be disputed as the funds are no longer in the scope
of program. However, it makes sense to lock the account if such scenario exists.
- Having a Trait exposed for the transaction engine grants the user of the Library
the flexibility to use any cache implementation.

### TODO Features
- [ ] Add support for AsyncRead for increased efficiency
- [ ] Add Typestate pattern to transaction for catching issues at compile time
