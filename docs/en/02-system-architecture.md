# System Architecture

## Modular Structure

The codebase is organized into the following primary modules:

- `node.rs`
- `network.rs`
- `block.rs`
- `transaction.rs`
- `wallet.rs`
- `main.rs`

This separation supports maintainability, testability, and clearer responsibility boundaries.

## Component Responsibilities

### Node Layer

`Node` stores and manages local state:

- local blockchain copy,
- local UTXO set,
- wallet state,
- local mempool,
- validator role flag.

It also performs transaction and block-level validation checks.

### Network Layer

`BlockchainNetwork` coordinates system-wide behavior:

- node creation and connectivity,
- transaction broadcasting,
- block propagation,
- validator selection,
- mempool-level conflict filtering.

### Block Layer

`Block` handles:

- transaction aggregation,
- Merkle root computation,
- hash generation,
- Proof of Work execution.

### Transaction Layer

`Transaction` defines:

- input/output structures,
- transaction identity generation,
- signing payload generation,
- fee computation,
- validity rules.

### Wallet Layer

`Wallet` is responsible for:

- ECDSA key generation,
- address derivation,
- signature generation and verification helpers,
- local UTXO and balance management.

## Data Flow Summary

1. A node creates and signs a transaction.
2. The network propagates the transaction.
3. The validator selects valid transactions from mempool.
4. A block is produced and mined.
5. The block is broadcast and local states are updated across nodes.
