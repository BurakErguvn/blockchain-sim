# Blockchain Simulation

This file is the English documentation entry point. Detailed technical content is organized in the modular documentation set under `docs/en/`.

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

## Minimal Run Command

```bash
cargo run
```

Additional run examples:

```bash
cargo run --bin sim_cli -- status
cargo run --bin api_server
```
