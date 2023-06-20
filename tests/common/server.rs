use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use assert_cmd::cargo::cargo_bin;
use tasklist::kill;

pub const MINER_ADDRESS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub struct Config {
    pub port: u16,
    pub peers: Vec<String>,
    pub peer_sync_ms: u64,
    pub max_blocks: u64,
    pub max_nonce: u64,
    pub difficulty: u32,
    pub transaction_waiting_ms: u64,
    pub miner_address: String,
}

pub struct ServerBuilder {
    config: Config,
}

#[allow(dead_code)]
impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        let config = Config {
            port: 8000,
            peer_sync_ms: 10,
            difficulty: 0,
            transaction_waiting_ms: 10,
            peers: Vec::<String>::new(),
            max_blocks: 0,
            max_nonce: 0,
            miner_address: MINER_ADDRESS.to_string(),
        };

        ServerBuilder { config }
    }

    pub fn difficulty(mut self, difficulty: u32) -> ServerBuilder {
        self.config.difficulty = difficulty;

        self
    }

    pub fn port(mut self, port: u16) -> ServerBuilder {
        self.config.port = port;

        self
    }

    pub fn peer(mut self, port: u16) -> ServerBuilder {
        let address = format!("http://localhost:{}", port);
        self.config.peers.push(address);

        self
    }

    pub fn start(self) -> Server {
        Server::new(self.config)
    }
}

type SyncedOutput = Arc<Mutex<Vec<String>>>;

pub struct Server {
    pub config: Config,
    process: Child,
    output: SyncedOutput,
}

#[allow(dead_code)]
impl Server {
    fn start_process(config: &Config) -> Child {
        Command::new(cargo_bin("rust_blockchain"))
            .env("PORT", config.port.to_string())
            .env("PEERS", config.peers.join(","))
            .env("DIFFICULTY", config.difficulty.to_string())
            .env(
                "TRANSACTION_WAITING_MS",
                config.transaction_waiting_ms.to_string(),
            )
            .env("PEER_SYNC_MS", config.peer_sync_ms.to_string())
            .env("MINER_ADDRESS", config.miner_address.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    }

    fn start_stdout_reading(process: &mut Child) -> SyncedOutput {
        let output = Arc::new(Mutex::new(Vec::<String>::new()));
        let thread_output = output.clone();
        let stdout = process.stdout.take().unwrap();

        thread::spawn(move || {
            let buf = BufReader::new(stdout);

            for line in buf.lines() {
                match line {
                    Ok(_) => {
                        thread_output.lock().unwrap().push(line.unwrap());
                    }

                    Err(_) => {
                        break;
                    }
                }
            }
        });

        output
    }

    fn search_message_in_output(&mut self, message: &str) -> bool {
        let lines = self.output.lock().unwrap();

        for line in lines.iter() {
            if line.contains(message) {
                return true;
            }
        }

        false
    }

    fn wait_for_log_message(&mut self, message: &str) {
        let wait_time = Duration::from_millis(50);
        let max_wait_time = Duration::from_millis(500);

        let start = Instant::now();

        println!("{}", message);

        while Instant::now() < start + max_wait_time {
            let message_was_found = self.search_message_in_output(message);

            if message_was_found {
                return;
            }

            thread::sleep(wait_time);
        }
    }

    pub fn new(config: Config) -> Server {
        let mut process = Server::start_process(&config);
        let output = Server::start_stdout_reading(&mut process);

        let mut server = Server {
            process,
            config,
            output,
        };

        server.wait_for_log_message("actix-web-service");

        server
    }

    pub fn wait_for_mining(&mut self) {
        self.wait_for_log_message("Valid block found for index");
    }

    pub fn wait_for_peer_sync(&mut self) {
        self.wait_for_log_message("Added new peer block");
    }

    pub fn wait_to_receive_block_in_api(&mut self) {
        self.wait_for_log_message("Received new block");
    }

    fn sleep_millis(millis: u64) {
        let wait_duration = Duration::from_millis(millis);

        thread::sleep(wait_duration);
    }

    fn wait_for_termination(&mut self) {
        let max_waiting_in_secs = 5;

        // check every second if the child has finished
        for _ in 0..max_waiting_in_secs {
            match self.process.try_wait().unwrap() {
                Some(_) => return,
                None => Server::sleep_millis(1000),
            }
        }

        // waited but child didn't finish, forcefully kill it
        let _ = self.process.kill();
        self.process.wait().unwrap();
    }

    fn stop(&mut self) {
        println!("Shutting down server on port {}", self.config.port);

        unsafe {
            let kill_process = kill(self.process.id());

            if !kill_process {
                println!("Kill process failed");
            }

            println!("Kill successful");
        }

        self.wait_for_termination();
    }
}

/**
 * Stopping the server on variable drop allows us to not worry about
 * leaving zombie child process in the background.
 * The Rust compiler ensures that this will be always be called no matter what (success or panic)
 * as soon as the variable is out of scope.
 */
impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}
