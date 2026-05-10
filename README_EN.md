# Blockchain Simulation

This file serves as the English documentation entry point. Detailed technical content has been moved to the modular documentation set under `docs/en/`.

## Documentation Entry

- [English Documentation Index](docs/en/README.md)

## Quick Links

- [Introduction and Scope](docs/en/01-introduction-and-scope.md)
- [System Architecture](docs/en/02-system-architecture.md)
- [Transaction and UTXO Model](docs/en/03-transaction-and-utxo-model.md)
- [Consensus and Security](docs/en/04-consensus-and-security.md)
- [Installation and Running](docs/en/05-installation-and-running.md)
- [Command-Line Interface Reference](docs/en/06-cli-reference.md)
- [Development Notes and Roadmap](docs/en/07-development-notes-and-roadmap.md)

## Recent Updates (Since the README Restructure)

- Added an **advanced mempool policy**: fixed `300 MB` capacity, minimum fee-rate admission, fee-rate-prioritized block selection, and RBF-lite replacement.
- Added **fork/reorg simulation**: work-score-based chain choice on equal lengths, deterministic tie-break, reorg depth tracking, and network-level scenario tests.
- Completed **Persistence Phase 1 and Phase 2**: disk save/load, startup recovery, schema migration (`v1 -> v2`), UTXO snapshots with checksum, and mempool persistence with revalidation.
- Added an **HTTP API program** (`cargo run --bin api_server`): health, network state, node/blockchain, mempool, transaction, and mining endpoints.
- Added a **next-generation CLI tool** (`cargo run --bin sim_cli`): stateful commands, REPL, history, autocomplete, typo suggestions, and scenario/alias/macro management.

## Minimal Run Command

```bash
cargo run
```

Additional run examples:

```bash
cargo run --bin sim_cli -- status
cargo run --bin api_server
```
