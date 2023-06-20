use std::{
    slice::Iter,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use thiserror::Error;

use super::{
    account_balance_map::AccountBalanceMap,
    block::{Block, BlockHash},
    transaction::Transaction,
};

pub type BlockVec = Vec<Block>;

type SyncedBlockVec = Arc<Mutex<BlockVec>>;
type SyncedAccountBalanceVec = Arc<Mutex<AccountBalanceMap>>;

pub const BLOCK_SUBSIDY: u64 = 100;

#[derive(Error, PartialEq, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum BlockchainError {
    #[error("Invalid index")]
    InvalidIndex,

    #[error("Invalid previous_hash")]
    InvalidPreviousHash,

    #[error("Invalid hash")]
    InvalidHash,

    #[error("Invalid difficulty")]
    InvalidDifficulty,

    #[error("Coinbase transaction not found")]
    CoinbaseTransactionNotFound,

    #[error("Invalid coinbase amount")]
    InvalidCoinbaseAmount,
}

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub difficulty: u32,
    blocks: SyncedBlockVec,
    account_balances: SyncedAccountBalanceVec,
}

impl Blockchain {
    fn create_genesis_block() -> Block {
        let index = 0;
        let nonce = 0;
        let previous_hash = BlockHash::default();
        let transactions = Vec::new();

        let mut block = Block::new(index, nonce, previous_hash, transactions);

        block.timestamp = 0;
        block.hash = block.calculate_hash();

        block
    }

    pub fn new(difficulty: u32) -> Blockchain {
        let genesis_block = Blockchain::create_genesis_block();

        let blocks = vec![genesis_block];
        let synced_blocks = Arc::new(Mutex::new(blocks));
        let synced_account_balances = SyncedAccountBalanceVec::default();

        Blockchain {
            difficulty,
            blocks: synced_blocks,
            account_balances: synced_account_balances,
        }
    }

    pub fn get_last_block(&self) -> Block {
        let blocks = self.blocks.lock().unwrap();

        blocks[blocks.len() - 1].clone()
    }

    pub fn get_all_blocks(&self) -> BlockVec {
        let blocks = self.blocks.lock().unwrap();

        blocks.clone()
    }

    fn process_coinbase(
        account_balances: &mut AccountBalanceMap,
        coinbase: Option<&Transaction>,
    ) -> Result<()> {
        let coinbase = match coinbase {
            Some(transaction) => transaction,
            None => return Err(BlockchainError::CoinbaseTransactionNotFound.into()),
        };

        let is_valid_amount = coinbase.amount == BLOCK_SUBSIDY;
        if !is_valid_amount {
            return Err(BlockchainError::InvalidCoinbaseAmount.into());
        }

        account_balances.add_amount(&coinbase.recipient, coinbase.amount);

        Ok(())
    }

    fn process_transfers(
        new_account_balances: &mut AccountBalanceMap,
        transaction_iter: Iter<Transaction>,
    ) -> Result<()> {
        for transaction in transaction_iter {
            new_account_balances.transfer(
                &transaction.sender,
                &transaction.recipient,
                transaction.amount,
            )?
        }

        Ok(())
    }

    fn calculate_new_account_balance(
        account_balances: &AccountBalanceMap,
        transactions: &[Transaction],
    ) -> Result<AccountBalanceMap> {
        let mut new_account_balances = account_balances.clone();
        let mut iter = transactions.iter();

        Blockchain::process_coinbase(&mut new_account_balances, iter.next())?;
        Blockchain::process_transfers(&mut new_account_balances, iter)?;

        Ok(new_account_balances)
    }

    fn udpate_account_balance(&self, transactions: &[Transaction]) -> Result<()> {
        let mut account_balances = self.account_balances.lock().unwrap();

        let new_account_balances =
            Blockchain::calculate_new_account_balance(&account_balances, transactions)?;

        *account_balances = new_account_balances;

        Ok(())
    }

    pub fn add_block(&self, block: Block) -> Result<()> {
        let mut blocks = self.blocks.lock().unwrap();
        let last = &blocks[blocks.len() - 1];

        if block.index != last.index + 1 {
            return Err(BlockchainError::InvalidIndex.into());
        }

        if block.previous_hash != last.hash {
            return Err(BlockchainError::InvalidPreviousHash.into());
        }

        if block.hash != block.calculate_hash() {
            return Err(BlockchainError::InvalidHash.into());
        }

        if block.hash.leading_zeros() < self.difficulty {
            return Err(BlockchainError::InvalidDifficulty.into());
        }

        self.udpate_account_balance(&block.transactions)?;

        blocks.push(block);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        account_balance_map::AccountBalanceMapError,
        address::{
            test_person_util::{person1, person2, person3},
            Address,
        },
    };

    use super::*;

    const NO_DIFFICULTY: u32 = 0;

    fn assert_err(result: Result<(), anyhow::Error>, error_type: BlockchainError) {
        let err = result.unwrap_err().downcast::<BlockchainError>().unwrap();
        assert_eq!(err, error_type);
    }

    fn assert_balance_err(result: Result<(), anyhow::Error>, error_type: AccountBalanceMapError) {
        let err = result
            .unwrap_err()
            .downcast::<AccountBalanceMapError>()
            .unwrap();
        assert_eq!(err, error_type);
    }

    #[test]
    fn should_have_valid_genesis_block() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let blocks = blockchain.get_all_blocks();
        assert_eq!(blocks.len(), 1);

        let block = blockchain.get_last_block();
        assert_eq!(block.hash, blocks[0].hash);

        assert_eq!(block.index, 0);
        assert_eq!(block.nonce, 0);
        assert_eq!(block.previous_hash, BlockHash::default());
        assert!(block.transactions.is_empty());
    }

    #[test]
    fn should_let_adding_valid_blocks() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let previous_hash = blockchain.get_last_block().hash;
        let coinbase = Transaction {
            sender: Address::default(),
            recipient: person2(),
            amount: BLOCK_SUBSIDY,
        };

        let transaction1 = Transaction {
            sender: person2(),
            recipient: person1(),
            amount: 5,
        };

        let transaction2 = Transaction {
            sender: person1(),
            recipient: person2(),
            amount: 5,
        };

        let block = Block::new(
            1,
            0,
            previous_hash,
            vec![coinbase, transaction1, transaction2],
        );

        let result = blockchain.add_block(block.clone());
        println!("ERROR: {:?}", result);
        assert!(result.is_ok());

        let blocks = blockchain.get_all_blocks();
        assert_eq!(blocks.len(), 2);

        let last_block = blockchain.get_last_block();
        assert_eq!(last_block.hash, block.hash);
    }

    #[test]
    fn should_not_let_adding_block_with_invalid_index() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let invalid_index = 2;
        let previous_hash = blockchain.get_last_block().hash;
        let block = Block::new(invalid_index, 0, previous_hash, Vec::new());

        let result = blockchain.add_block(block.clone());
        assert_err(result, BlockchainError::InvalidIndex);
    }

    #[test]
    fn should_not_let_adding_block_with_invalid_previous_hash() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let invalid_previous_hash = BlockHash::default();
        let block = Block::new(1, 0, invalid_previous_hash, Vec::new());

        let result = blockchain.add_block(block.clone());
        assert_err(result, BlockchainError::InvalidPreviousHash);
    }

    #[test]
    fn should_not_led_adding_block_with_invalid_hash() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let previous_hash = blockchain.get_last_block().hash;
        let mut block = Block::new(1, 0, previous_hash, Vec::new());
        block.hash = BlockHash::default();

        let result = blockchain.add_block(block.clone());
        assert_err(result, BlockchainError::InvalidHash);
    }

    #[test]
    fn should_not_let_adding_block_with_invalid_difficulty() {
        let difficulty: u32 = 30;
        let blockchain = Blockchain::new(difficulty);

        let previous_hash = blockchain.get_last_block().hash;
        let block = Block::new(1, 0, previous_hash, Vec::new());

        assert!(block.hash.leading_zeros() < difficulty);

        let result = blockchain.add_block(block.clone());
        assert_err(result, BlockchainError::InvalidDifficulty);
    }

    #[test]
    fn should_not_let_adding_block_with_no_coinbase() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let previous_hash = blockchain.get_last_block().hash;
        let block = Block::new(1, 0, previous_hash, vec![]);

        let result = blockchain.add_block(block.clone());
        assert_err(result, BlockchainError::CoinbaseTransactionNotFound);
    }

    #[test]
    fn should_not_let_adding_block_with_invalid_coinbase() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let previous_hash = blockchain.get_last_block().hash;
        let coinbase = Transaction {
            sender: Address::default(),
            recipient: Address::default(),
            amount: BLOCK_SUBSIDY + 1,
        };

        let block = Block::new(1, 0, previous_hash, vec![coinbase]);

        let result = blockchain.add_block(block.clone());
        assert_err(result, BlockchainError::InvalidCoinbaseAmount)
    }

    #[test]
    fn should_not_let_add_transaction_with_insufficient_funds() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let previous_hash = blockchain.get_last_block().hash;
        let coinbase = Transaction {
            sender: Address::default(),
            recipient: person2(),
            amount: BLOCK_SUBSIDY,
        };

        let invalid_transaction = Transaction {
            sender: person2(),
            recipient: person1(),
            amount: BLOCK_SUBSIDY + 1,
        };

        let block = Block::new(1, 0, previous_hash, vec![coinbase, invalid_transaction]);

        let result = blockchain.add_block(block.clone());
        assert_balance_err(result, AccountBalanceMapError::InsufficientFunds);
    }

    #[test]
    fn should_not_let_add_transaction_with_non_existent_sender() {
        let blockchain = Blockchain::new(NO_DIFFICULTY);

        let previous_hash = blockchain.get_last_block().hash;

        let coinbase = Transaction {
            sender: Address::default(),
            recipient: person2(),
            amount: BLOCK_SUBSIDY,
        };

        let invalid_transaction = Transaction {
            sender: person3(),
            recipient: person2(),
            amount: 1,
        };

        let block = Block::new(1, 0, previous_hash, vec![coinbase, invalid_transaction]);

        let result = blockchain.add_block(block.clone());
        assert_balance_err(result, AccountBalanceMapError::SenderAccountDoesNotExist);
    }
}
