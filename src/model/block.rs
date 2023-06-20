use chrono::Utc;
use crypto::{digest::Digest, sha2::Sha256};
use ethereum_types::U256;
use serde::{Deserialize, Serialize};

use super::transaction::Transaction;

pub type BlockHash = U256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub nonce: u64,
    pub previous_hash: BlockHash,
    pub hash: BlockHash,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn calculate_hash(&self) -> BlockHash {
        let mut hashable_data = self.clone();
        hashable_data.hash = BlockHash::default();

        let serialized = serde_json::to_string(&hashable_data).unwrap();

        let mut byte_hash = <[u8; 32]>::default();
        let mut hasher = Sha256::new();

        hasher.input_str(&serialized);
        hasher.result(&mut byte_hash);

        U256::from(byte_hash)
    }

    pub fn new(
        index: u64,
        nonce: u64,
        previous_hash: BlockHash,
        transactions: Vec<Transaction>,
    ) -> Block {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp_millis(),
            nonce,
            previous_hash,
            hash: BlockHash::default(),
            transactions,
        };

        block.hash = block.calculate_hash();

        block
    }
}
