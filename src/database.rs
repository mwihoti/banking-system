// SPDX-License-Identifier: Unlicense

use std::path::PathBuf;

use crate::luhn::AccountNumber;

use rand::prelude::*;
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct Account {
    pub id: u64,
    pub account_number: String,
    pub balance: u64,
    pub pin: String,
}
fn database_path() -> PathBuf {
    PathBuf::from("bank.s3db")
}

pub fn initialise_bankdb() -> Result<Connection> {
    let db = Connection::open(database_path())?;

    let command = " CREATE TABLE IF NOT EXISTS account(
    id INTEGER  PRIMARY KEY,
    account_number TEXT,
    pin TEXT DEFAULT '000000',
    balance INTEGER DEFAULT 0
)";
db.execute(command, ())?;
ok(db)
}