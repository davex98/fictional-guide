use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Error {
    InsufficientFunds,
    LockedAccount,
}

pub struct AccountsRepository {
    accounts: HashMap<u16, Account>,
}

impl AccountsRepository {
    pub fn new() -> AccountsRepository {
        AccountsRepository {
            accounts: Default::default(),
        }
    }

    pub fn get_or_create(&mut self, id: u16) -> &mut Account {
        self.accounts.entry(id).or_insert_with(|| Account::new(id))
    }

    pub fn display_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());

        let mut sorted: Vec<(&u16, &Account)> = self.accounts.iter().collect();
        sorted.sort_by_key(|(_, c)| c.client_id());
        for (_, client) in &sorted {
            wtr.serialize(client)?;
        }
        wtr.flush()?;

        Ok(())
    }
}

impl Default for AccountsRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Account {
    client_id: u16,
    available_balance: f64,
    held_balance: f64,
    total_balance: f64,
    locked: bool,
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut account = serializer.serialize_struct("Account", 5)?;
        account.serialize_field("client", &self.client_id)?;
        account.serialize_field(
            "available",
            &((self.available_balance * 10000.0).round() / 10000.0),
        )?;
        account.serialize_field("held", &((self.held_balance * 10000.0).round() / 10000.0))?;
        account.serialize_field("total", &((self.total_balance * 10000.0).round() / 10000.0))?;
        account.serialize_field("locked", &self.locked)?;
        account.end()
    }
}

impl Account {
    pub fn new(client_id: u16) -> Account {
        Account {
            client_id,
            available_balance: 0.0,
            held_balance: 0.0,
            total_balance: 0.0,
            locked: false,
        }
    }

    pub fn client_id(&self) -> u16 {
        self.client_id
    }

    fn is_locked(&self) -> Result<(), Error> {
        if self.locked {
            return Err(Error::LockedAccount);
        }

        Ok(())
    }

    fn has_sufficient_funds(&self, amount: f64) -> Result<(), Error> {
        if amount > self.available_balance {
            return Err(Error::InsufficientFunds);
        }

        Ok(())
    }

    pub fn deposit(&mut self, amount: f64) -> Result<(), Error> {
        self.is_locked()?;
        self.available_balance += amount;
        self.total_balance += amount;
        Ok(())
    }

    pub fn withdrawal(&mut self, amount: f64) -> Result<(), Error> {
        self.is_locked()?;
        self.has_sufficient_funds(amount)?;
        self.available_balance -= amount;
        self.total_balance -= amount;
        Ok(())
    }

    pub fn dispute(&mut self, amount: f64) -> Result<(), Error> {
        self.is_locked()?;
        self.has_sufficient_funds(amount)?;
        self.available_balance -= amount;
        self.held_balance += amount;
        Ok(())
    }

    fn has_sufficient_hold_balande(&self, amount: f64) -> Result<(), Error> {
        if amount > self.held_balance {
            return Err(Error::InsufficientFunds);
        }

        Ok(())
    }
    pub fn resolve(&mut self, amount: f64) -> Result<(), Error> {
        self.is_locked()?;
        self.has_sufficient_hold_balande(amount)?;
        self.held_balance -= amount;
        self.available_balance += amount;
        Ok(())
    }

    pub fn chargeback(&mut self, amount: f64) -> Result<(), Error> {
        self.is_locked()?;
        self.has_sufficient_hold_balande(amount)?;
        self.held_balance -= amount;
        self.total_balance -= amount;
        self.locked = true;
        Ok(())
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    #[cfg(test)]
    pub fn available_balance(&self) -> f64 {
        (self.available_balance * 10000.0).round() / 10000.0
    }
    #[cfg(test)]
    pub fn held_balance(&self) -> f64 {
        (self.held_balance * 10000.0).round() / 10000.0
    }
    #[cfg(test)]
    pub fn total_balance(&self) -> f64 {
        (self.total_balance * 10000.0).round() / 10000.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn base_account() -> Account {
        Account::new(1)
    }

    fn base_account_with_funds(funds: f64) -> Account {
        let mut acc = Account::new(1);
        acc.available_balance += funds;
        acc.total_balance += funds;
        acc
    }

    #[test]
    fn deposit() {
        let mut account = base_account();
        assert!(account.deposit(1.88889).is_ok());
        assert_eq!(account.available_balance(), 1.8889);
        assert_eq!(account.total_balance(), 1.8889);
    }

    #[test]
    fn debit_no_funds() {
        let mut account = base_account();
        let result = account.withdrawal(2.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InsufficientFunds);
        assert_eq!(account.available_balance(), 0.0);
        assert_eq!(account.total_balance(), 0.0);
    }

    #[test]
    fn debit_too_much() {
        let mut account = base_account_with_funds(19.0);
        let result = account.withdrawal(50.9);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InsufficientFunds);
        assert_eq!(account.available_balance(), 19.0);
        assert_eq!(account.total_balance(), 19.0);
    }

    #[test]
    fn debit() {
        let mut account = base_account_with_funds(19.0);
        assert!(account.withdrawal(10.9).is_ok());
        assert_eq!(account.available_balance(), 8.1);
        assert_eq!(account.total_balance(), 8.1);
    }

    #[test]
    fn hold() {
        let mut account = base_account_with_funds(19.0);

        account
            .dispute(10.0)
            .expect("Should have been able to hold funds");
        assert_eq!(account.held_balance(), 10.0);
        assert_eq!(account.available_balance(), 9.0);
        assert_eq!(account.total_balance(), 19.0);
    }

    #[test]
    fn hold_no_funds() {
        let mut account = base_account_with_funds(1.0);

        let result = account.dispute(10.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InsufficientFunds);
        assert_eq!(account.held_balance(), 0.0);
        assert_eq!(account.available_balance(), 1.0);
        assert_eq!(account.total_balance(), 1.0);
    }

    #[test]
    fn release() {
        let mut account = base_account_with_funds(19.0);

        account
            .dispute(10.0)
            .expect("Should have been able to hold funds");
        assert_eq!(account.held_balance(), 10.0);
        assert_eq!(account.available_balance(), 9.0);
        assert_eq!(account.total_balance(), 19.0);
        account
            .resolve(10.0)
            .expect("Should have been able to release funds");
        assert_eq!(account.held_balance(), 0.0);
        assert_eq!(account.available_balance(), 19.0);
        assert_eq!(account.total_balance(), 19.0);
    }

    #[test]
    fn release_no_funds() {
        let mut account = base_account_with_funds(19.0);
        let result = account.resolve(10.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InsufficientFunds);
        assert_eq!(account.held_balance(), 0.0);
        assert_eq!(account.available_balance(), 19.0);
        assert_eq!(account.total_balance(), 19.0);
    }

    #[test]
    fn chargeback() {
        let mut account = base_account_with_funds(20.0);
        assert!(account.dispute(10.0).is_ok());
        assert!(account.chargeback(10.0).is_ok());
        assert!(account.locked);

        let result = account.deposit(10.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::LockedAccount);
    }
}
