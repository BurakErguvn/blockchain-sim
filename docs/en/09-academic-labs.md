# Academic Laboratory Guide

This document explains how to use the pedagogical modules under `labs/` in a classroom setting.

## Setup

```bash
cargo run --bin sim_cli -- --profile classroom config validate
cargo run --bin sim_cli -- --profile classroom lab list
```

The `config/classroom.toml` profile provides:

- 3 nodes
- `difficulty = 1`
- short block time
- local API bind (`127.0.0.1`)
- dedicated state file (`./data/classroom_state.json`)

## Instructor flow

```bash
cargo run --bin sim_cli -- --profile classroom lab show 01-utxo-and-transfers
cargo run --bin sim_cli -- --profile classroom lab run 01-utxo-and-transfers --json
```

`lab run` executes setup, demo steps, and assertions in one pass. Use it for CI and pre-class smoke checks.

## Student flow

1. Read the module `README.md`.
2. Prepare the environment:

```bash
cargo run --bin sim_cli -- --profile classroom lab setup 01-utxo-and-transfers
```

3. Follow the hands-on commands (`tx`, `mempool`, `mine`, `nodes`, `chain`).
4. Answer `questions.md`.
5. Run the automated grader when applicable:

```bash
cargo run --bin sim_cli -- --profile classroom lab run 01-utxo-and-transfers --json
```

> Note: Demo-flag checks (signature tamper, double-spend, reorg depth) require `lab run`. `lab verify` only inspects persisted state and does not retain in-memory demo flags.

## Grading

Shared rubric notes: [labs/shared/grading.md](../labs/shared/grading.md)

## Module index

See [labs/README.md](../labs/README.md).
