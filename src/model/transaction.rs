use serde::{Deserialize, Serialize};

use super::address::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
}
