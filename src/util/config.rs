extern crate dotenv;

use std::{env, str::FromStr};

use dotenv::dotenv;

use crate::model::Address;

type StringVec = Vec<String>;

pub struct Config {
    // Network settings
    pub port: u16,

    // Peer settings
    pub peers: StringVec,
    pub peer_sync_ms: u64,

    // Miner settings
    pub max_blocks: u64,
    pub max_nonce: u64,
    pub difficulty: u32,
    pub transaction_waiting_ms: u64,
    pub miner_address: Address,
}

impl Config {
    pub fn read_envvar<T: FromStr>(key: &str, default_value: T) -> T {
        match env::var(key) {
            Ok(value) => value.parse::<T>().unwrap_or(default_value),
            Err(_) => default_value,
        }
    }

    pub fn read_vec_envvar(key: &str, separator: &str, default_value: StringVec) -> StringVec {
        match env::var(key) {
            Ok(value) => value
                .trim()
                .split_terminator(separator)
                .map(str::to_string)
                .collect(),
            Err(_) => default_value,
        }
    }

    pub fn read() -> Config {
        dotenv().ok();

        Config {
            // Network settings
            port: Config::read_envvar::<u16>("PORT", 8000),

            // Peer settings
            peers: Config::read_vec_envvar("PEERS", ",", StringVec::default()),
            peer_sync_ms: Config::read_envvar("PEER_SYNC_MS", 10000),

            // Miner settings
            max_blocks: Config::read_envvar("MAX_BLOCKS", 0),
            max_nonce: Config::read_envvar("MAX_NONCE", 1_000_000),
            difficulty: Config::read_envvar("DIFFICULTY", 10),
            transaction_waiting_ms: Config::read_envvar("TRANSACTION_WAITING_MS", 10000),
            miner_address: Config::read_envvar("MINER_ADDRESS", Address::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }

    #[test]
    fn read_present_envvar() {
        let var_name = "PRESENT_ENVVAR";
        let real_value = 9000;
        env::set_var(var_name, real_value.to_string());

        let default_value = 8000 as u16;
        let value = Config::read_envvar::<u16>(var_name, default_value);

        assert_eq!(value, real_value);

        env::remove_var(var_name);
    }

    #[test]
    fn read_present_vec_envvar() {
        let var_name = "PRESENT_VEC_ENVVAR";
        let value = "FOO,BAR";
        env::set_var(var_name, value.to_string());

        let default_value = StringVec::default();
        let actual_value = Config::read_vec_envvar(var_name, ",", default_value);
        let expected_value: Vec<String> = value.split(",").map(str::to_string).collect();

        assert!(do_vecs_match(&actual_value, &expected_value));

        env::remove_var(var_name);
    }

    #[test]
    fn read_non_present_envvar() {
        let var_name = "NON_PRESENT_ENVVAR";

        env::remove_var(var_name);

        let default_value = 8000 as u16;
        let value = Config::read_envvar::<u16>(var_name, default_value);
        assert_eq!(value, default_value);

        let default_vec_value = StringVec::default();
        let vec_value = Config::read_vec_envvar(var_name, ",", default_vec_value.clone());
        assert_eq!(&vec_value, &default_vec_value);
    }

    #[test]
    fn read_invalid_envvar() {
        let var_name = "INVALID=VAR=NAME";

        let default_value = 8000 as u16;
        let value = Config::read_envvar::<u16>(var_name, default_value);
        assert_eq!(value, default_value);

        let default_vec_value = StringVec::default();
        let vec_value = Config::read_vec_envvar(var_name, ",", default_vec_value.clone());
        assert!(do_vecs_match(&vec_value, &default_vec_value));
    }
}
