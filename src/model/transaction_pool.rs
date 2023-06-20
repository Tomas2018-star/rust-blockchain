use std::sync::{Arc, Mutex};

use log::info;

use super::transaction::Transaction;

pub type TransactionVec = Vec<Transaction>;

type SyncedTransactionVec = Arc<Mutex<TransactionVec>>;

#[derive(Debug, Clone)]
pub struct TransactionPool {
    transaction: SyncedTransactionVec,
}

impl TransactionPool {
    pub fn new() -> TransactionPool {
        TransactionPool {
            transaction: SyncedTransactionVec::default(),
        }
    }

    pub fn add_transaction(&self, transaction: Transaction) {
        let mut transactions = self.transaction.lock().unwrap();
        transactions.push(transaction);
        info!("Transaction added");
    }

    pub fn pop(&self) -> TransactionVec {
        let mut transactions = self.transaction.lock().unwrap();
        let transactions_clone = transactions.clone();
        transactions.clear();

        transactions_clone
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        address::test_person_util::{person1, person2},
        transaction::Transaction,
    };

    use super::TransactionPool;

    fn create_mock_transaction(amount: u64) -> Transaction {
        Transaction {
            sender: person1(),
            recipient: person2(),
            amount,
        }
    }

    #[test]
    fn should_be_empty_after_creation() {
        let transaction_pool = TransactionPool::new();

        let transactions = transaction_pool.pop();
        assert!(transactions.is_empty());
    }

    #[test]
    fn should_pop_single_value() {
        let transaction_pool = TransactionPool::new();

        let transaction = create_mock_transaction(1);
        transaction_pool.add_transaction(transaction.clone());

        let mut transactions = transaction_pool.pop();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].amount, transaction.amount);

        transactions = transaction_pool.pop();
        assert!(transactions.is_empty());
    }

    #[test]
    fn should_pop_multiple_values() {
        let transaction_pool = TransactionPool::new();

        let transaction_a = create_mock_transaction(1);
        let transaction_b = create_mock_transaction(2);
        transaction_pool.add_transaction(transaction_a.clone());
        transaction_pool.add_transaction(transaction_b.clone());

        let mut transactions = transaction_pool.pop();
        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].amount, transaction_a.amount);
        assert_eq!(transactions[1].amount, transaction_b.amount);

        transactions = transaction_pool.pop();
        assert!(transactions.is_empty());
    }
}
