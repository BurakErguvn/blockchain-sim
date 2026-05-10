# Command-Line Interface Reference

The project now exposes two CLI surfaces:

1. **Legacy interactive CLI** (`cargo run`)  
2. **Next-generation stateful CLI tool** (`cargo run --bin sim_cli`)

This document primarily focuses on `sim_cli`.

---

## 1) Next-Generation CLI (`sim_cli`)

### 1.1 Basic Invocation

```bash
cargo run --bin sim_cli -- <command>
```

Global flags:

- `--state-path <file>`: state file path (default: `./data/network_state.json`)
- `--json`: output in JSON format

Example:

```bash
cargo run --bin sim_cli -- --json status
```

### 1.2 Command Groups

#### A) `init`

Bootstraps a new network and mines a genesis block.

```bash
cargo run --bin sim_cli -- init --nodes 5 --difficulty 2 --block-time 60
```

Optional flag:

- `--force`: overwrite an existing state file

#### B) `status`

Returns network summary (node count, validator, tip, mempool size).

```bash
cargo run --bin sim_cli -- status
```

#### C) `nodes`

- `nodes list`
- `nodes show <id>`

```bash
cargo run --bin sim_cli -- nodes list
cargo run --bin sim_cli -- nodes show 2
```

#### D) `chain`

- `chain tip`
- `chain show --node-id <id> [--limit N]`

```bash
cargo run --bin sim_cli -- chain tip
cargo run --bin sim_cli -- chain show --node-id 0 --limit 5
```

#### E) `tx`

- `tx create --sender-id <id> --recipient-id <id> --amount-coin <value>`

```bash
cargo run --bin sim_cli -- tx create --sender-id 0 --recipient-id 1 --amount-coin 1.25
```

#### F) `mempool`

- `mempool list`

```bash
cargo run --bin sim_cli -- mempool list
```

#### G) `mine`

- `mine once`

```bash
cargo run --bin sim_cli -- mine once
```

#### H) `persistence`

- `persistence info`
- `persistence save [--path <file>]`
- `persistence load --path <file>`

```bash
cargo run --bin sim_cli -- persistence info
cargo run --bin sim_cli -- persistence save --path ./data/backup.json
```

### 1.3 REPL Mode

If no command is provided, REPL starts automatically:

```bash
cargo run --bin sim_cli --
```

Alternative:

```bash
cargo run --bin sim_cli -- repl
```

REPL capabilities:

- persistent history (`.sim_cli_history`),
- autocomplete (root + subcommand level),
- typo suggestions,
- built-in `help`, `exit`, `quit` commands.

### 1.4 Scenarios, Aliases, and Macros (Phase 3)

#### Scenarios

```bash
cargo run --bin sim_cli -- scenario list
cargo run --bin sim_cli -- scenario run quickstart
cargo run --bin sim_cli -- scenario run fork-reorg --primary 0 --secondary 1
```

#### Aliases

```bash
cargo run --bin sim_cli -- alias add st status
cargo run --bin sim_cli -- alias list
cargo run --bin sim_cli -- alias remove st
```

#### Macros

```bash
cargo run --bin sim_cli -- macro add demo --cmd "status" --cmd "mempool list"
cargo run --bin sim_cli -- macro run demo
cargo run --bin sim_cli -- macro remove demo
```

REPL shortcut:

```text
!demo
```

---

## 2) Legacy Interactive CLI (`cargo run`)

The original command set is still available:

- `bakiye <node_id>`
- `transfer <sender_id> <receiver_id> <amount>`
- `durum`
- `blockchain <node_id>`
- `mempool`
- `çıkış` (`exit`, `quit`)

Legacy mode is preserved for backward compatibility; `sim_cli` is recommended for operational workflows.
