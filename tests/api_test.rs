mod common;

use crate::common::{
    Api, Block, BlockHash, ServerBuilder, Transaction, BLOCK_SUBSIDY, MINER_ADDRESS, PERSON1,
    PERSON2,
};
use serial_test::serial;

#[test]
#[serial]
#[cfg(windows)]
fn test_should_get_a_valid_genesis_block() {
    let node = ServerBuilder::new().start();
    let blocks = node.get_blocks();

    assert_eq!(blocks.len(), 1);

    let genesis_block = blocks.first().unwrap();

    assert_eq!(genesis_block.index, 0);
    assert_eq!(genesis_block.nonce, 0);
    assert_eq!(genesis_block.previous_hash, BlockHash::default());
    assert!(genesis_block.transactions.is_empty());
}

#[test]
#[serial]
#[cfg(windows)]
fn test_should_let_add_transactions() {
    let mut node = ServerBuilder::new().start();
    let genesis_block = node.get_last_block();

    let transaction = Transaction {
        sender: MINER_ADDRESS.to_string(),
        recipient: PERSON2.to_string(),
        amount: 10 as u64,
    };
    let res = node.add_transaction(&transaction);

    assert_eq!(res.status().as_u16(), 200);

    node.wait_for_mining();

    let blocks = node.get_blocks();

    assert_eq!(blocks.len(), 2);

    let mined_block = blocks.last().unwrap();

    assert_eq!(mined_block.index, 1);
    assert_eq!(mined_block.previous_hash, genesis_block.hash);
    assert_eq!(mined_block.transactions.len(), 2);

    let mined_transaction = mined_block.transactions.last().unwrap();
    assert_eq!(*mined_transaction, transaction);
}

#[test]
#[serial]
#[cfg(windows)]
fn test_should_let_add_valid_block() {
    let node = ServerBuilder::new().start();
    let genesis_block = node.get_last_block();

    let coinbase = Transaction {
        sender: PERSON1.to_string(),
        recipient: PERSON1.to_string(),
        amount: BLOCK_SUBSIDY,
    };

    let valid_block = Block {
        index: 1,
        timestamp: 0,
        nonce: 0,
        previous_hash: genesis_block.hash,
        hash: BlockHash::default(),
        transactions: vec![coinbase],
    };

    let res = node.add_block(&valid_block);

    assert_eq!(res.status().as_u16(), 200);
}

#[test]
#[serial]
#[cfg(windows)]
fn test_should_not_let_add_invalid_block() {
    let node = ServerBuilder::new().start();

    let invalid_block = Block {
        index: 0,
        timestamp: 0,
        nonce: 0,
        previous_hash: BlockHash::default(),
        hash: BlockHash::default(),
        transactions: [].to_vec(),
    };

    let res = node.add_block(&invalid_block);

    assert_eq!(res.status().as_u16(), 400);
}
