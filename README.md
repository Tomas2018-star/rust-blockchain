# rust-blockchain

An educational purpose, Proof of Work blockchain written in Rust.

Features:

- Provides REST API to retrieve blocks and add transactions.
- Synchronize new blocks with peer nodes.
- Mine new blocks with Proof of Work algorithm with a fixed difficulty.

## Getting Started

You will need Rust and Cargo installed.

```bash
# Run all tests
$ cargo test

# Build the project in release node
$ cargo build --release

# Run the application
$ ./target/release/rust_blockchain
```

The application will start listening and mining on a default port `8000` for client requests via REST API. To change any environment variables like port, difficulty, etc. please refer to `.env.example` and create a `.env` file with your preferred environment variables.

## Client REST API

The application provides a REST API for clients to operate with the blockchain.

| Method | URL           | Description                          |
| ------ | ------------- | ------------------------------------ |
| GET    | /blocks       | List all blocks of the blockchain    |
| POST   | /blocks       | Append a new block to the blockchain |
| POST   | /transactions | Add a new transaction to the pool    |

### Sample Request

```json
{
  "name": "Get all blocks of the blockchain",
  "request": {
    "url": "http://localhost:8000/blocks",
    "method": "GET",
    "header": [],
    "body": {
      "mode": "raw",
      "raw": ""
    },
    "description": ""
  },
  "response": []
}
```

```json
{
  "name": "Add a new block",
  "request": {
    "url": "http://localhost:8000/blocks",
    "method": "POST",
    "header": [
      {
        "key": "Content-Type",
        "value": "application/json"
      }
    ],
    "body": {
      "mode": "raw",
      "raw": "{\n    \"index\": 1,\n    \"timestamp\": 0,\n    \"nonce\": 0,\n    \"previous_hash\": \"0x0\",\n    \"hash\": \"0x0\",\n    \"transactions\": [\n        {\n            \"sender\": \"0\",\n            \"recipient\": \"1\",\n            \"amount\": 1000\n        },\n        {\n            \"sender\": \"0\",\n            \"recipient\": \"2\",\n            \"amount\": 1000\n        }\n    ]\n}"
    },
    "description": ""
  },
  "response": []
}
```

```json
{
  "name": "Add a new transaction to the pool",
  "request": {
    "url": "http://localhost:8000/transactions",
    "method": "POST",
    "header": [
      {
        "key": "Content-Type",
        "value": "application/json",
        "description": ""
      }
    ],
    "body": {
      "mode": "raw",
      "raw": "{\n    \"sender\": \"f780b958227ff0bf5795ede8f9f7eaac67e7e06666b043a400026cbd421ce28e\",\n    \"recipient\": \"51df097c03c0a6e64e54a6fce90cb6968adebd85955917ed438e3d3c05f2f00f\",\n    \"amount\": 1002\n}"
    },
    "description": ""
  },
  "response": []
}
```

## Block Structure

Each block contains the following data:

- **index**: position of the block in the blockchain
- **timestamp**: date and time of block creation
- **nonce**: arbitrary number that makes the block, when hashed, meet the mining difficulty restriction. Is the number that miners are competing to get first
- **previous_hash**: hash of the previous block in the chain. Allows to maintain order of blocks in the blockchain. There is an exception with the first block of the chain (genesis block) which has no previous_hash
- **hash**: hash of the block including all fields
- **transactions**: a list of all transactions included in the block. Each transaction has a **sender**, **recipient** and **amount**.

### Concurrency implementation

In this project, the `main` thread spawns three OS threads:

- One for the **miner**. As mining is very computationally-intensive, we want a dedicated OS thread to not slow down other operations in the application. In a real blockchain we would also want parallel mining (by handling a different subrange of nonces in each thread), but for simplicity we will only use one thread.
- Other thread for the **REST API**. The API uses [`actix-web`](https://github.com/actix/actix-web), which internally uses [`tokio`](https://crates.io/crates/tokio), so it's optimized for asynchronous operations.
- A thread for the **peer system**, that periodically sends and receives new blocks from peers over the network.

Thread spawning and handling is implemented using [`crossbeam-utils`](https://crates.io/crates/crossbeam-utils) to reduce boilerplate code from the standard library.

Also, all threads share data, specifically the **block list** and the **transaction pool**. Those two data structures are implemented by using `Arc<Mutex>` to allow multiple concurrent writes and reads in a safe way from separate threads.

# Credit

This project is written with reference to https://github.com/mrnaveira/rust-blockchain for self-learning purposes.
