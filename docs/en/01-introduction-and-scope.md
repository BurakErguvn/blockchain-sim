# Introduction and Scope

## Objective

This project is a Rust-based simulation designed to explain the core operational principles of blockchain systems. The implementation integrates distributed node behavior, block production, transaction validation, and UTXO-based balance accounting in an educational setting.

## Scope Boundaries

The project is intended for instructional and experimental use rather than production deployment. Therefore:

- The networking layer is represented as a simulation model.
- Consensus behavior is designed for conceptual clarity.
- The command-line interface focuses on observability and learning.

## Core Concepts

- **Node:** A participant that stores a local copy of the chain and communicates with other nodes.
- **Validator:** A node that is temporarily authorized to produce a block.
- **Block:** A data structure containing transactions, linkage metadata, timestamp, nonce, and hash fields.
- **UTXO:** An unspent transaction output used as the atomic unit of spendable value.
- **Proof of Work:** A mechanism requiring the block hash to satisfy a difficulty constraint.

## Intended Usage

This project is suitable for:

- academic study of blockchain fundamentals,
- systems programming exercises in Rust,
- practical observation of transaction validation and UTXO dynamics.
