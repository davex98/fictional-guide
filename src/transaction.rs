use serde::Deserialize;
use std::collections::HashMap;

#[derive(Copy, Debug, Clone, PartialOrd, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Transaction {
    r#type: Type,
    #[serde(rename(deserialize = "client"))]
    account_id: u16,
    #[serde(rename(deserialize = "tx"))]
    id: u32,
    #[serde(default)]
    amount: Option<f64>,
    #[serde(skip_deserializing)]
    is_dispute: bool,
}

impl Transaction {
    pub fn new(id: u32, r#type: Type, account_id: u16, amount: f64) -> Transaction {
        Transaction {
            id,
            r#type,
            account_id,
            amount: Some(amount),
            is_dispute: false,
        }
    }

    pub fn r#type(&self) -> Type {
        self.r#type
    }

    pub fn amount(&self) -> f64 {
        self.amount.unwrap()
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn account_id(&self) -> u16 {
        self.account_id
    }

    pub fn is_dispute(&self) -> bool {
        self.is_dispute
    }
}

pub struct TransactionLedger {
    transactions: HashMap<u32, Transaction>,
}
impl Default for TransactionLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionLedger {
    pub fn new() -> TransactionLedger {
        TransactionLedger {
            transactions: Default::default(),
        }
    }
    pub fn append(&mut self, tx: &Transaction) {
        self.transactions.entry(tx.id).or_insert(*tx);
    }

    pub fn get(&self, tx_id: u32) -> Option<&Transaction> {
        self.transactions.get(&tx_id)
    }

    pub fn dispute_tx(&mut self, tx_id: u32) {
        let tx = self.transactions.get_mut(&tx_id);
        tx.unwrap().is_dispute = true;
    }

    pub fn undispute_tx(&mut self, tx_id: u32) {
        let tx = self.transactions.get_mut(&tx_id);
        tx.unwrap().is_dispute = false;
    }
}
