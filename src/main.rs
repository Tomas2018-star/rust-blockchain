use util::initialize_logger;

use crate::{
    api::Api,
    miner::Miner,
    model::{Blockchain, TransactionPool},
    peer::Peer,
    util::{execution, termination, Config, Context},
};

#[macro_use]
extern crate log;

mod api;
mod miner;
mod model;
mod peer;
mod util;

fn main() {
    initialize_logger();

    info!("Starting up");

    termination::set_ctrlc_handler();

    let config = Config::read();
    let difficulty = config.difficulty;

    let context = Context {
        config,
        blockchain: Blockchain::new(difficulty),
        pool: TransactionPool::new(),
    };

    let miner = Miner::new(&context);
    let api = Api::new(&context);
    let peer = Peer::new(&context);

    execution::run_in_parallel(vec![&miner, &api, &peer]);
}
