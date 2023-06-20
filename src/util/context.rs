use crate::model::{Blockchain, TransactionPool};

use super::config::Config;

pub struct Context {
    pub config: Config,
    pub blockchain: Blockchain,
    pub pool: TransactionPool,
}
