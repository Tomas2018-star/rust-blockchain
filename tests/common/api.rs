use ethereum_types::U256;
use isahc::{Body, ReadResponseExt, Request, Response};
use serde::{Deserialize, Serialize};

use super::Server;

pub type BlockHash = U256;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub nonce: u64,
    pub previous_hash: BlockHash,
    pub hash: BlockHash,
    pub transactions: Vec<Transaction>,
}

pub const PERSON1: &str = "f780b958227ff0bf5795ede8f9f7eaac67e7e06666b043a400026cbd421ce28e";
pub const PERSON2: &str = "51df097c03c0a6e64e54a6fce90cb6968adebd85955917ed438e3d3c05f2f00f";

pub const BLOCK_SUBSIDY: u64 = 100;

pub trait Api {
    fn get_blocks(&self) -> Vec<Block>;
    fn get_last_block(&self) -> Block;
    fn add_block(&self, block: &Block) -> Response<Body>;
    fn add_valid_block(&self) -> Response<Body>;
    fn add_transaction(&self, transaction: &Transaction) -> Response<Body>;
}

impl Api for Server {
    fn get_blocks(&self) -> Vec<Block> {
        let uri = format!("{}/blocks", get_base_url(self));
        let mut response = isahc::get(uri).unwrap();

        assert_eq!(response.status().as_u16(), 200);

        let raw_body = response.text().unwrap();
        let blocks: Vec<Block> = serde_json::from_str(&raw_body).unwrap();

        blocks
    }

    fn get_last_block(&self) -> Block {
        self.get_blocks().last().unwrap().to_owned()
    }

    fn add_valid_block(&self) -> Response<Body> {
        let last_block = self.get_last_block();

        let coinbase = Transaction {
            sender: PERSON1.to_string(),
            recipient: PERSON2.to_string(),
            amount: BLOCK_SUBSIDY,
        };

        let valid_block = Block {
            index: last_block.index + 1,
            timestamp: 0,
            nonce: 0,
            previous_hash: last_block.hash,
            hash: BlockHash::default(),
            transactions: vec![coinbase],
        };

        self.add_block(&valid_block)
    }

    fn add_block(&self, block: &Block) -> Response<Body> {
        let uri = format!("{}/blocks", get_base_url(self));
        let body = serde_json::to_string(&block).unwrap();

        post_request(uri, body)
    }

    fn add_transaction(&self, transaction: &Transaction) -> Response<Body> {
        let uri = format!("{}/transactions", get_base_url(self));
        let body = serde_json::to_string(&transaction).unwrap();

        post_request(uri, body)
    }
}

fn get_base_url(server: &Server) -> String {
    format!("http://localhost:{}", server.config.port)
}

fn post_request(uri: String, body: String) -> Response<Body> {
    let request = Request::post(uri)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();

    isahc::send(request).unwrap()
}
