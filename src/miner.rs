use anyhow::Result;
use thiserror::Error;

use crate::{
    model::{
        Address, Block, BlockHash, Blockchain, Transaction, TransactionPool, TransactionVec,
        BLOCK_SUBSIDY,
    },
    util::{
        execution::{sleep_millis, Runnable},
        Context,
    },
};

#[derive(Error, Debug)]
pub enum MinerError {
    #[error("No valid block was mined at index `{0}`")]
    BlockNotMined(u64),
}

pub struct Miner {
    miner_address: Address,
    max_blocks: u64,
    max_nonce: u64,
    transaction_waiting_ms: u64,
    blockchain: Blockchain,
    pool: TransactionPool,
    target: BlockHash,
}

impl Runnable for Miner {
    fn run(&self) -> Result<()> {
        self.start()
    }
}

impl Miner {
    fn create_target(difficulty: u32) -> BlockHash {
        BlockHash::MAX >> difficulty
    }

    pub fn new(context: &Context) -> Miner {
        let target = Self::create_target(context.config.difficulty);

        Miner {
            miner_address: context.config.miner_address.clone(),
            max_blocks: context.config.max_blocks,
            max_nonce: context.config.max_nonce,
            transaction_waiting_ms: context.config.transaction_waiting_ms,
            blockchain: context.blockchain.clone(),
            pool: context.pool.clone(),
            target,
        }
    }

    fn must_stop_mining(&self, block_counter: u64) -> bool {
        self.max_blocks > 0 && block_counter >= self.max_blocks
    }

    fn create_coinbase_transaction(&self) -> Transaction {
        Transaction {
            sender: Address::default(),
            recipient: self.miner_address.clone(),
            amount: BLOCK_SUBSIDY,
        }
    }

    fn create_next_block(
        &self,
        last_block: &Block,
        transactions: TransactionVec,
        nonce: u64,
    ) -> Block {
        let index = (last_block.index + 1) as u64;
        let previous_hash = last_block.hash;

        Block::new(index, nonce, previous_hash, transactions)
    }

    fn mine_block(&self, last_block: &Block, transactions: &TransactionVec) -> Option<Block> {
        let coinbase = self.create_coinbase_transaction();
        let mut block_transactions = transactions.clone();
        block_transactions.insert(0, coinbase);

        for nonce in 0..self.max_nonce {
            let next_block = self.create_next_block(last_block, block_transactions.clone(), nonce);

            if next_block.hash < self.target {
                return Some(next_block);
            }
        }

        None
    }

    pub fn start(&self) -> Result<()> {
        info!("Start mining with dificulty {}", self.blockchain.difficulty);

        let mut block_counter = 0;

        loop {
            if self.must_stop_mining(block_counter) {
                info!("Block limit reached, stopping mining");

                return Ok(());
            }

            let transactions = self.pool.pop();

            if transactions.is_empty() {
                sleep_millis(self.transaction_waiting_ms);

                continue;
            }

            let last_block = self.blockchain.get_last_block();
            let mining_result = self.mine_block(&last_block, &transactions.clone());

            match mining_result {
                Some(block) => {
                    info!("Valid block found for index {}", block.index);
                    self.blockchain.add_block(block.clone())?;
                    block_counter += 1;
                }

                None => {
                    let index = last_block.index + 1;
                    error!("No valid block was found for index {}", index);

                    return Err(MinerError::BlockNotMined(index).into());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::test_person_util::{person1, person2};

    use super::*;

    const MAX_DIFFICULTY: u32 = 256;

    fn miner_address() -> Address {
        person1()
    }

    fn create_miner(difficulty: u32, max_nonce: u64) -> Miner {
        let miner_address = miner_address();
        let max_blocks = 1;
        let transaction_waiting_ms = 1;
        let target = Miner::create_target(difficulty);

        let blockchain = Blockchain::new(difficulty);
        let pool = TransactionPool::new();

        Miner {
            miner_address,
            max_blocks,
            max_nonce,
            transaction_waiting_ms,
            blockchain,
            pool,
            target,
        }
    }

    fn create_default_miner() -> Miner {
        let difficulty = 1;
        let max_nonce = 1;

        create_miner(difficulty, max_nonce)
    }

    fn create_empty_block() -> Block {
        return Block::new(0, 0, BlockHash::default(), Vec::new());
    }

    #[test]
    fn test_create_next_block() {
        let miner = create_default_miner();
        let block = create_empty_block();

        let next_block = miner.create_next_block(&block, Vec::new(), 0);

        assert_eq!(next_block.index, block.index + 1);
        assert_eq!(next_block.previous_hash, block.hash);
    }

    #[test]
    fn test_create_target_valid_difficulty() {
        for difficulty in 0..MAX_DIFFICULTY {
            let target = Miner::create_target(difficulty);
            assert_eq!(target.leading_zeros(), difficulty);
        }
    }

    #[test]
    fn test_create_target_overflowing_difficulty() {
        let target = Miner::create_target(MAX_DIFFICULTY + 1);
        assert_eq!(target.leading_zeros(), MAX_DIFFICULTY);
    }

    fn assert_mined_block_is_valid(mined_block: &Block, previous_block: &Block, difficulty: u32) {
        assert_eq!(mined_block.index, previous_block.index + 1);
        assert_eq!(mined_block.previous_hash, previous_block.hash);
        assert!(mined_block.hash.leading_zeros() >= difficulty as u32);
    }

    #[test]
    fn test_mine_block_found() {
        let difficulty = 1;
        let max_nonce = 1_000;

        let miner = create_miner(difficulty, max_nonce);
        let last_block = create_empty_block();
        let result = miner.mine_block(&last_block, &Vec::new());
        assert!(result.is_some());

        let mined_block = result.unwrap();
        assert_mined_block_is_valid(&mined_block, &last_block, difficulty);
    }

    #[test]
    fn test_mine_block_not_found() {
        let difficulty = MAX_DIFFICULTY;
        let max_nonce = 10;

        let miner = create_miner(difficulty, max_nonce);
        let last_block = create_empty_block();
        let result = miner.mine_block(&last_block, &Vec::new());
        assert!(result.is_none());
    }

    fn add_mock_transaction(pool: &TransactionPool) {
        let transaction = Transaction {
            sender: miner_address(),
            recipient: person2(),
            amount: 3,
        };

        pool.add_transaction(transaction.clone());
    }

    #[test]
    fn test_run_block_found() {
        let difficulty = 1;
        let max_nonce = 1_000_000;

        let miner = create_miner(difficulty, max_nonce);
        let blockchain = miner.blockchain.clone();
        let pool = miner.pool.clone();
        add_mock_transaction(&pool);

        let result = miner.run();
        assert!(result.is_ok());

        let blocks = blockchain.get_all_blocks();
        assert_eq!(blocks.len(), 2);

        let genesis_block = &blocks[0];
        let mined_block = &blocks[1];
        assert_mined_block_is_valid(mined_block, genesis_block, blockchain.difficulty);

        let mined_transactions = &mined_block.transactions;
        assert_eq!(mined_transactions.len(), 2);

        let transactions = pool.pop();
        assert!(transactions.is_empty());
    }

    #[test]
    #[should_panic(expected = "No valid block was mined at index `1`")]
    fn test_run_block_not_found() {
        let difficulty = MAX_DIFFICULTY;
        let max_nonce = 1;

        let miner = create_miner(difficulty, max_nonce);
        let pool = &miner.pool;
        add_mock_transaction(pool);

        // should return BlockNotMined error
        miner.run().unwrap();
    }
}
