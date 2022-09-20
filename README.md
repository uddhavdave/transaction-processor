# TRANSACTION ENGINE
## Introduction
This program provides functionality to process transaction history of client's 
asset account and output account details after processing all transactions.

## Usage
Program's CLI interface accepts csv filename as arguments and outputs the csv data on
STDOUT. The output can be piped to a csv file for storage purposes.
Following is a sample command:
`$ cargo run -- transaction.csv > accounts.csv`

## Features and Design Decisions
Basic Transaction types supported are:
- Deposit
- Withdraw
Special transaction types supported are:
- Dispute: Flags a transaction and holds the amount
- Resolve: Removes flag and deposits amount back to client's asset account
- Chargeback: Removes flag and credits amount to client's credit accounts
