# Blockchain Simulation

Hello and welcome. This project is a community-friendly blockchain simulation that brings block production, transaction validation, UTXO tracking, mempool policies, persistence, API, and an advanced CLI experience into a single learning and experimentation environment.

You can think of this README as a quick starting point: it helps you understand where to begin, routes you to the right documents, and makes contributing easier. Deeper technical details are maintained in the modular documentation set under `docs/en/`.

If you prefer Turkish documentation, see: [README_TR.md](README_TR.md)

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
- [HTTP API Usage Guide](docs/en/08-http-api-usage.md)
- [Academic Labs](docs/en/09-academic-labs.md)
- [Guided Student Mode](docs/en/10-guided-student-mode.md)

## Minimal Run Command

```bash
cargo run
```

Additional run examples:

```bash
cargo run --bin sim_cli -- status
cargo run --bin api_server
cargo run --bin sim_cli -- --profile classroom lab list
cargo run --bin sim_cli -- --profile classroom lab run 01-utxo-and-transfers --json
```

Academic lab package: [labs/README.md](labs/README.md)
