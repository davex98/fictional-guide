use crate::account::AccountsRepository;
use crate::transaction::{Transaction, TransactionLedger, Type};

pub struct Engine<'a> {
    pub tx_ledger: &'a mut TransactionLedger,
    pub accounts: &'a mut AccountsRepository,
}

impl Engine<'_> {
    pub fn new<'a>(
        tx_ledger: &'a mut TransactionLedger,
        accounts: &'a mut AccountsRepository,
    ) -> Engine<'a> {
        Engine {
            tx_ledger,
            accounts,
        }
    }

    fn deposit(&mut self, tx: &Transaction) {
        let account = self.accounts.get_or_create(tx.account_id());
        if self.tx_ledger.get(tx.id()).is_some() {
            return;
        }
        if let Err(err) = account.deposit(tx.amount()) {
            log::warn!("could not deposit money: {:?}", err)
        }
    }

    fn withdrawal(&mut self, tx: &Transaction) {
        let account = self.accounts.get_or_create(tx.account_id());
        if self.tx_ledger.get(tx.id()).is_some() {
            return;
        }
        if let Err(err) = account.withdrawal(tx.amount()) {
            log::warn!("could not withdrawal money: {:?}", err)
        }
    }

    fn dispute(&mut self, tx: &Transaction) {
        let account = self.accounts.get_or_create(tx.account_id());
        if let Some(old_tx) = self.tx_ledger.get(tx.id()) {
            if old_tx.is_dispute() || account.client_id() != old_tx.account_id() {
                return;
            }
            if let Err(err) = account.dispute(old_tx.amount()) {
                log::warn!("could not dispute transaction: {:?}", err);
                return;
            }
            self.tx_ledger.dispute_tx(tx.id())
        }
    }

    fn resolve(&mut self, tx: &Transaction) {
        let account = self.accounts.get_or_create(tx.account_id());
        match self.tx_ledger.get(tx.id()) {
            None => (),
            Some(old_tx) => {
                if old_tx.is_dispute() && old_tx.account_id() == account.client_id() {
                    if let Err(err) = account.resolve(old_tx.amount()) {
                        log::warn!("could not resolve: {:?}", err);
                        return;
                    }
                    self.tx_ledger.undispute_tx(tx.id());
                }
            }
        }
    }

    fn chargeback(&mut self, tx: &Transaction) {
        let account = self.accounts.get_or_create(tx.account_id());
        match self.tx_ledger.get(tx.id()) {
            None => {}
            Some(tx) => {
                if tx.is_dispute() && tx.account_id() == account.client_id() {
                    if let Err(err) = account.chargeback(tx.amount()) {
                        log::warn!("could not chargeback money: {:?}", err)
                    }
                }
            }
        }
    }

    pub fn process(&mut self, input_tx: &[Transaction]) {
        for tx in input_tx {
            match tx.r#type() {
                Type::Deposit => self.deposit(tx),
                Type::Withdrawal => self.withdrawal(tx),
                Type::Dispute => self.dispute(tx),
                Type::Resolve => self.resolve(tx),
                Type::Chargeback => self.chargeback(tx),
            }

            self.tx_ledger.append(tx)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::transaction::Type;

    #[test]
    fn deposit() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [Transaction::new(1, Type::Deposit, 1, 5.0)];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        assert_eq!(account.available_balance(), 5.0);
    }

    #[test]
    fn withdrawal() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.0),
            Transaction::new(2, Type::Withdrawal, 1, 2.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        assert_eq!(account.available_balance(), 3.0);
    }

    #[test]
    fn withdrawal_with_insufficient() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.0),
            Transaction::new(2, Type::Withdrawal, 1, 6.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        assert_eq!(account.available_balance(), 5.0);
    }

    #[test]
    fn dispute() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.0),
            Transaction::new(2, Type::Deposit, 1, 3.0),
            Transaction::new(2, Type::Dispute, 1, 0.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        let tx = tx_ledger.get(2).unwrap();
        assert_eq!(account.available_balance(), 5.0);
        assert_eq!(account.held_balance(), 3.0);
        assert_eq!(account.total_balance(), 8.0);
        assert!(tx.is_dispute());
    }

    #[test]
    fn resolve() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.0),
            Transaction::new(2, Type::Deposit, 1, 3.0),
            Transaction::new(2, Type::Dispute, 1, 0.0),
            Transaction::new(2, Type::Resolve, 1, 0.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        assert_eq!(account.available_balance(), 8.0);
        assert_eq!(account.held_balance(), 0.0);
        assert_eq!(account.total_balance(), 8.0);
    }

    #[test]
    fn resolve_with_different_account_id() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.0),
            Transaction::new(2, Type::Deposit, 1, 3.0),
            Transaction::new(2, Type::Dispute, 1, 0.0),
            Transaction::new(2, Type::Resolve, 2, 0.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        assert_eq!(account.available_balance(), 5.0);
        assert_eq!(account.held_balance(), 3.0);
        assert_eq!(account.total_balance(), 8.0);
    }

    #[test]
    fn chargeback() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.0),
            Transaction::new(2, Type::Deposit, 1, 3.0),
            Transaction::new(2, Type::Dispute, 1, 0.0),
            Transaction::new(2, Type::Chargeback, 1, 0.0),
            Transaction::new(1, Type::Deposit, 1, 5.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        assert_eq!(account.available_balance(), 5.0);
        assert_eq!(account.held_balance(), 0.0);
        assert_eq!(account.total_balance(), 5.0);
        assert!(account.locked());
    }

    #[test]
    fn dispute_with_different_account_id() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.0),
            Transaction::new(2, Type::Deposit, 1, 3.0),
            Transaction::new(2, Type::Dispute, 2, 0.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        let tx = tx_ledger.get(2).unwrap();
        assert_eq!(account.available_balance(), 8.0);
        assert_eq!(account.held_balance(), 0.0);
        assert_eq!(account.total_balance(), 8.0);
        assert!(!tx.is_dispute());
    }

    #[test]
    fn dispute_two_times() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 1.77),
            Transaction::new(2, Type::Deposit, 1, 1.77),
            Transaction::new(3, Type::Deposit, 1, 1.77),
            Transaction::new(1, Type::Dispute, 1, 0.0),
            Transaction::new(1, Type::Dispute, 1, 0.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        let tx = tx_ledger.get(1).unwrap();
        assert_eq!(account.available_balance(), 3.54);
        assert_eq!(account.held_balance(), 1.77);
        assert_eq!(account.total_balance(), 5.31);
        assert!(tx.is_dispute());
    }

    #[test]
    fn withdrawal_the_same_tx_twice() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.00),
            Transaction::new(2, Type::Withdrawal, 1, 2.0),
            Transaction::new(2, Type::Withdrawal, 1, 2.0),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        assert_eq!(account.available_balance(), 3.00);
        assert_eq!(account.total_balance(), 3.00);
    }

    #[test]
    fn deposite_the_same_tx_twice() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.00),
            Transaction::new(1, Type::Deposit, 1, 5.00),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        assert_eq!(account.available_balance(), 5.00);
        assert_eq!(account.total_balance(), 5.00);
    }

    #[test]
    fn dispute_the_same_tx_twice() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.00),
            Transaction::new(1, Type::Dispute, 1, 0.00),
            Transaction::new(1, Type::Dispute, 1, 0.00),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        let tx = tx_ledger.get(1).unwrap();
        assert_eq!(account.available_balance(), 0.00);
        assert_eq!(account.held_balance(), 5.00);
        assert_eq!(account.total_balance(), 5.00);
        assert!(tx.is_dispute());
    }

    #[test]
    fn resolve_the_same_tx_twice() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.00),
            Transaction::new(2, Type::Deposit, 1, 5.00),
            Transaction::new(1, Type::Dispute, 1, 0.00),
            Transaction::new(1, Type::Resolve, 1, 0.00),
            Transaction::new(2, Type::Resolve, 1, 0.00),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        let tx = tx_ledger.get(1).unwrap();
        assert_eq!(account.available_balance(), 10.00);
        assert_eq!(account.held_balance(), 0.00);
        assert_eq!(account.total_balance(), 10.00);
        assert!(!tx.is_dispute());
    }

    #[test]
    fn resolve_the_same_tx_with_diff_acc() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.00),
            Transaction::new(2, Type::Deposit, 1, 5.00),
            Transaction::new(1, Type::Dispute, 1, 0.00),
            Transaction::new(1, Type::Resolve, 2, 0.00),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        let tx = tx_ledger.get(1).unwrap();
        assert_eq!(account.available_balance(), 5.00);
        assert_eq!(account.held_balance(), 5.00);
        assert_eq!(account.total_balance(), 10.00);
        assert!(tx.is_dispute());
    }

    #[test]
    fn chargeback_the_same_tx_with_diff_acc() {
        let mut acc_repo = AccountsRepository::new();
        let mut tx_ledger = TransactionLedger::new();
        let mut engine = Engine::new(&mut tx_ledger, &mut acc_repo);
        let transactions = [
            Transaction::new(1, Type::Deposit, 1, 5.00),
            Transaction::new(2, Type::Deposit, 1, 5.00),
            Transaction::new(1, Type::Dispute, 1, 0.00),
            Transaction::new(1, Type::Chargeback, 2, 0.00),
        ];
        engine.process(&transactions);
        let account = acc_repo.get_or_create(1);
        let tx = tx_ledger.get(1).unwrap();
        assert_eq!(account.available_balance(), 5.00);
        assert_eq!(account.held_balance(), 5.00);
        assert_eq!(account.total_balance(), 10.00);
        assert!(tx.is_dispute());
    }
}
