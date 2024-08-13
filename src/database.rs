// SPDX-License-Identifier: Unlicense

use std::{path::PathBuf};
use crate::luhn::AccountNumber;
use rand::prelude::*;
use rusqlite::{Connection, Result as SqlResult};

#[derive(Debug)]
pub struct Account {
    pub id: u64,
    pub account_number: String,
    pub balance: u64,
    pub pin: String,
}

#[cfg(not(test))]
fn database_path() -> PathBuf {
	PathBuf::from("bank.s3db")
}

#[cfg(test)]
fn database_path() -> PathBuf {
	PathBuf::from("mock_bank.s3db")
}

pub fn initialise_bankdb() -> SqlResult<Connection> {
    let db = Connection::open(database_path())?;

    let command = "CREATE TABLE IF NOT EXISTS account(
        id INTEGER PRIMARY KEY,
        account_number TEXT,
        pin TEXT DEFAULT '000000',
        balance INTEGER DEFAULT 0
    )";
    db.execute(command, ())?;
    Ok(db)
}

/// Creating and storing accounts
pub fn create_account(data: &AccountNumber, balance: u64) -> SqlResult<()> {
    let db = initialise_bankdb()?;
    let account_number = data.to_string();
    
    let mut stmt = db.prepare("SELECT id, account_number, balance, pin FROM account")?;
    let accounts = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            account_number: row.get(1)?,
            balance: row.get(2)?,
            pin: row.get(3)?,
        })
    })?;
    
    let get_latest_max_id = {
        let mut x = 0;
        for account in accounts.flatten() {
            if account.id > x {
                x = account.id;
            }
        }
        x
    };

    let newest_max_id = get_latest_max_id + 1;
    let mut rng = thread_rng();
    let pin: String = (0..6).map(|_| rng.gen_range(0..=9).to_string()).collect();

    let new_account = Account {
        id: newest_max_id,
        account_number,
        balance,
        pin,
    };

    db.execute(
        "INSERT INTO account (id, account_number, pin, balance) VALUES (?1, ?2, ?3, ?4)", 
        (
            &new_account.id,
            &new_account.account_number,
            &new_account.pin,
            &new_account.balance,
        ),
    )?;
    Ok(())
}

/// Depositing money into a currently active account
pub fn deposit(amount: &str, pin: &str, account_number: &str) -> SqlResult<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account WHERE account_number='{}';",
        account_number
    );
    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;
    let correct_pin = pin_from_db == pin;

    if correct_pin {
        db.execute(
            "UPDATE account SET balance = balance + ?1 WHERE account_number=?2",
            (amount, account_number),
        )?;
        let query_string = format!(
            "SELECT balance FROM account WHERE account_number='{}';",
            account_number
        );
        let amount_from_db: u64 = db.query_row(&query_string, [], |row| row.get(0))?;
        println!(
            "The account number `{}` now has a balance of `{}`.\n",
            &account_number, &amount_from_db
        );
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}

/// Transferring money between accounts from a currently active account
pub fn transfer(
    amount: &str,
    pin: &str,
    account_number1: &str,
    account_number2: &str,
) -> SqlResult<()> {
    if account_number1 == account_number2 {
        eprintln!("Cannot perform a transfer to the same account!");
        return Ok(());
    }

    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin, balance FROM account WHERE account_number='{}';",
        account_number1
    );

    let (pin_from_db, balance): (String, u64) = db.query_row(&query_string, [], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    let correct_pin = pin_from_db == pin;
    if correct_pin {
        let amount = amount
            .parse::<u64>()
            .expect("Not able to parse string to u64");

        if amount > balance {
            eprintln!(
                "You are trying to transfer an amount that exceeds your current balance... aborting...\n"
            );
        } else {
            // Add money to account 2
            db.execute(
                "UPDATE account SET balance = balance + ?1 WHERE account_number=?2",
                (amount, account_number2),
            )?;
            // Subtract money from account 1
            db.execute(
                "UPDATE account SET balance = balance - ?1 WHERE account_number=?2",
                (amount, account_number1),
            )?;
            let query_string = format!(
                "SELECT balance FROM account WHERE account_number='{}';",
                account_number1
            );

            let updated_balance: u64 = db.query_row(&query_string, [], |row| row.get(0))?;

            println!(
                "The account number `{}` now has a balance of `{}`.\n",
                &account_number1, &updated_balance
            );
        }
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}

/// Withdrawing money from a currently active account
pub fn withdraw(amount: &str, pin: &str, account_number: &str) -> SqlResult<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin, balance FROM account WHERE account_number='{}';",
        account_number
    );

    let (pin_from_db, balance): (String, u64) = db.query_row(&query_string, [], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    let correct_pin = pin_from_db == pin;

    if correct_pin {
        let amount = amount
            .parse::<u64>()
            .expect("Not able to parse string to u64");

        if amount > balance {
            eprintln!(
                "You are trying to withdraw an amount that exceeds your current deposit... aborting...\n"
            );
        } else {
            db.execute(
                "UPDATE account SET balance = balance - ?1 WHERE account_number=?2",
                (amount, account_number),
            )?;

            let query_string = format!(
                "SELECT balance FROM account WHERE account_number='{}';",
                account_number
            );

            let updated_balance: u64 = db.query_row(&query_string, [], |row| row.get(0))?;

            println!(
                "The account number `{}` now has a balance of `{}`.\n",
                &account_number, &updated_balance
            );
        }
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}

/// Deleting a currently active account
pub fn delete_account(account_number: &str, pin: &str) -> SqlResult<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account WHERE account_number='{}';",
        account_number
    );

    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;

    let correct_pin = pin_from_db == pin;

    if correct_pin {
        db.execute(
            "DELETE FROM account WHERE account_number=?1",
            [account_number],
        )?;
        println!("DELETED ACCOUNT: {}", &account_number);
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}

/// Showing the current balance of a currently active account
pub fn show_balance(account_number: &str) -> SqlResult<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT balance FROM account WHERE account_number='{}';",
        account_number
    );

    let amount_from_db: u64 = db.query_row(&query_string, [], |row| row.get(0))?;

    println!(
        "The account number `{}` now has a balance of `{}`.\n",
        &account_number, &amount_from_db
    );
    Ok(())
}
fn fetch_account(account: &str) -> Result<Account> {
	let db = initialise_bankdb()?;
	let mut stmt = db.prepare("SELECT id, account_number, balance, pin FROM account")?;
	let accounts = stmt.query_map([], |row| {
    	Ok(Account {
        	id: row.get(0)?,
        	account_number: row.get(1)?,
        	balance: row.get(2)?,
        	pin: row.get(3)?,
    	})
	})?;

	let accounts = accounts.flatten().find(|acc| acc.account_number == account);
	if let Some(fetched_account) = accounts {
    		Ok(fetched_account)
	} else {
    		Err(rusqlite::Error::QueryReturnedNoRows)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn created_account_is_correct_fetched_from_db() -> Result<()> {
    		let acc1 = Account::new()?;
    		let acc2 = fetch_account(&acc1.account_number)?;

    		assert_eq!(acc1.id, acc2.id);

    		Ok(())
	}
}