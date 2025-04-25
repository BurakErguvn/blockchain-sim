# Blockchain Simulation

This project is a Rust-based application that simulates the basic operation of a blockchain network. It was developed to demonstrate the fundamental features and proof of concept of a real blockchain system.

## Features

### Node Structure
- Each node maintains a copy of the blockchain
- Validators can create new blocks
- There are connections and communication between nodes
- Each node has a unique identifier (ID)
- Nodes cannot connect to themselves

### Block Structure
- Index: The sequence number of the block in the chain
- Timestamp: The time stamp when the block was created
- Data: Data stored in the block (transactions)
- Previous Hash: Hash value of the previous block
- Hash: SHA-256 hash of the current block
- Nonce: Counter used for the Proof of Work algorithm

### Blockchain Features
- Genesis Block: The first block of the chain
- Immutability: Block data cannot be changed, changes are detected
- Consensus: Validation by majority rule
- Proof of Work: Simulates the difficulty required for mining
- Distributed Ledger: Each node keeps a copy of the entire blockchain
- Temporary Validator Authority: Validators' permissions are revoked after creating a single block
- Mining Result Hash Distribution: Nodes receive the hash resulting from Proof of Work, not the transaction hash

### Security Features
- SHA-256 hash algorithm usage
- Block verification mechanism
- Manipulation detection and correction system
- Majority-based consensus mechanism
- System to prevent power concentration in a single validator

## How It Works

1. **Network Creation**:
   - Various nodes are created and connected to each other (not to themselves)
   - Initially, each node contains a Genesis block

2. **Validator Selection**:
   - A random node is selected as a validator
   - Only validators can create new blocks
   - Each validator can only create one block, then their authority is revoked

3. **Transaction Creation and Mining**:
   - A new transaction is created
   - The validator receives and processes this transaction (creates SHA-256 hash)
   - A new block is created with the Proof of Work algorithm (requiring a specific number of leading zeros)
   - The hash value resulting from block mining (hash with nonce) is distributed to the entire network
   - The new block is broadcast to all nodes in the network
   - The validator's authority is revoked

4. **Security and Validation**:
   - Nodes continuously check the integrity of the blockchain
   - Manipulation attempts are detected
   - Corrupted blockchains are corrected by majority rule

5. **Consensus Algorithm**:
   - The majority of nodes in the network determine the valid chain
   - When a node is manipulated, other nodes take corrective actions

## Simulation Scenarios

The simulation includes the following scenarios:

1. **Normal Transaction Flow**:
   - A validator is selected and adds a new transaction
   - Block mining is performed and added to the chain
   - The validator's authority is revoked

2. **Hash Manipulation**:
   - A regular node attempts to change the hash
   - The consensus mechanism detects and rejects this

3. **Blockchain Manipulation**:
   - A node attempts to manipulate blockchain data
   - After changing the data, it calculates new nonce and hash according to PoW rules
   - Other nodes detect the manipulation (even if PoW rules are satisfied despite data changes)
   - Manipulation is prevented by majority consensus and the chain is corrected

## Technical Details

### Technologies Used

- **Programming Language**: Rust
- **Hash Algorithm**: SHA-256 (sha2 crate)
- **Random Number Generator**: rand crate

### Project Structure

- **src/main.rs**: Main simulation flow and test scenarios
- **src/node.rs**: Node structure and related implementations
- **src/block.rs**: Block structure and related functions
- **src/network.rs**: BlockchainNetwork structure and related functions
- **LICENSE**: MIT license (Copyright 2024 Burak Ergüven)
- **README.md**: Project documentation (Turkish)
- **README_EN.md**: Project documentation (English)

### Features Included

- **Decentralized**: Distributed structure among nodes
- **Transparent**: All nodes can see the blockchain
- **Secure**: SHA-256 hash and verification mechanisms
- **Immutable**: Changes are detected and corrected
- **Democratic**: No node can have permanent control

## How to Run

1. Make sure Rust and Cargo are installed
2. Clone the project
3. Go to the project directory in Terminal
4. Run the following command:

```bash
cargo run
```

## Future Developments

- Smart contract support
- More sophisticated P2P network simulation
- Digital signature support for transactions

## Recent Updates

### Modular Structure Improvements

- **Block Structure Separation**: Block structure and related implementations were moved to `block.rs` file for a more modular structure.
- **Network Structure Separation**: BlockchainNetwork structure and related implementations were moved to `network.rs`.
- **Code Organization**: The project's code organization was improved, with each structure placed in its own file.
- **Comprehensive Block Validation System**: Blockchain validity is now based not only on content control but also on chain validation and hash comparisons.
- **Simulation Improvement**: The main simulation now creates 5 transactions for a more realistic blockchain, followed by manipulation attempts.

---

This project is an educational simulation developed to understand and learn the basic principles of blockchain technology. 