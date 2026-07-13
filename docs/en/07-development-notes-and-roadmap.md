# Development Notes and Roadmap

## Detailed Recent Changes (After README Restructure)

This section captures the major technical additions introduced after the modular README transition.

### 1) Advanced Mempool Policy and Fee Mechanics

- Mempool capacity is now strictly capped at `300 MB`.
- Minimum fee-rate admission was introduced (sat/kB threshold).
- Mempool entries now track policy metadata:
  - estimated size,
  - fee amount,
  - fee-rate,
  - arrival sequence.
- Block assembly now prioritizes mempool entries by fee-rate.
- RBF-lite behavior was added:
  - a conflicting transaction can replace an existing one,
  - only if it satisfies the configured minimum fee-rate increase.
- Post-sync and post-broadcast mempool reconciliation has been hardened.

### 2) Fork/Reorg Simulation and Chain Selection

- Chain selection for equal-length candidates now uses:
  1. chain length,
  2. cumulative work score,
  3. deterministic tip-hash tie-break.
- Common-ancestor detection and reorg-depth estimation were added.
- A network-level fork/reorg simulation path was introduced (`simulate_fork_and_reorg`).
- Integration tests now verify node state and mempool consistency across reorg transitions.

### 3) Persistence Phase 1 + Phase 2

#### Phase 1
- Network metadata, node chains, and wallet identity are now persisted on disk.
- Crash-safe write flow was introduced:
  - write temp file,
  - `fsync`,
  - atomic rename.
- Startup recovery now restores persisted state when available.

#### Phase 2
- Schema versioning and migration support (`v1 -> v2`) was added.
- Per-node UTXO snapshots with checksums were introduced.
- Safe fallback behavior on checksum mismatch:
  - rebuild UTXO state directly from blockchain history.
- Mempool persistence and policy-based revalidation on restart were added.

### 4) HTTP API Program

- A dedicated API binary was added (`api_server`).
- Core endpoints were introduced for:
  - health checks,
  - network state,
  - node list/detail,
  - blockchain inspection,
  - mempool inspection,
  - transaction creation,
  - single-step block mining.
- Persistence hooks now run after API write operations.

### 5) Next-Generation CLI (Phases 1-2-3)

#### Phase 1
- New `sim_cli` binary with stateful subcommands:
  - init/status/nodes/chain/tx/mempool/mine/persistence.
- Global runtime flags:
  - `--state-path`,
  - `--json`.

#### Phase 2
- Interactive REPL mode (default when no command is provided).
- Persistent command history.
- Command suggestions and initial autocomplete support.

#### Phase 3
- Scenario command set:
  - `scenario list`,
  - `scenario run quickstart`,
  - `scenario run fork-reorg`.
- Alias management:
  - add/list/remove + REPL expansion.
- Macro management:
  - add/list/remove/run + `!macro` shortcut execution.
- Namespace-aware autocomplete at root and subcommand levels.

### 6) Configuration System (Phases A-B-C)

#### Phase A
- A centralized `Settings` model was introduced.
- `config/default.toml` became the single baseline for app/network/persistence/API parameters.
- Startup flows for `main`, `api_server`, and `sim_cli` were integrated with this settings model.

#### Phase B
- A multi-source resolution chain was added:
  1. CLI overrides,
  2. environment,
  3. profile file (`config/<profile>.toml`),
  4. default file (`config/default.toml`),
  5. code defaults.
- Field-level environment overrides were added.
- Config tests were expanded to cover profile/env/precedence behaviors.

#### Phase C
- New `sim_cli config` command namespace:
  - `config show`,
  - `config paths`,
  - `config validate`.
- REPL autocomplete/help output was updated for the config namespace.

### 7) CI/CD Quality Pipeline (Phases 1-2-3)

#### Phase 1: Baseline quality gate
- GitHub Actions CI checks:
  - `cargo fmt --all -- --check`,
  - `cargo clippy --all-targets --all-features`,
  - `cargo test`.

#### Phase 2: Dependency security
- Added `cargo audit`.
- Added `cargo deny check advisories bans`.
- Added `deny.toml` policy definitions for advisories/bans/sources.

#### Phase 3: Release + smoke
- Added `release.yml` for tag-driven release build, packaging, and checksum generation.
- Added CLI smoke checks in CI:
  - `config validate`,
  - `config paths`.

### 8) Academic Laboratory Package

- Added 6 pedagogical modules under `labs/` (UTXO, signatures, mempool, PoW, forks/reorgs, attacks).
- Each module includes student guide, questions, rubric, instructor notes, and a `lab.toml` manifest.
- Added `config/classroom.toml` classroom profile.
- Added `sim_cli lab` command group:
  - `lab list|show|setup|run|verify`
- File-driven lab runner supports automated assertion grading with JSON output.
- Added `tx create --fee-satoshi` for fee-controlled mempool experiments.
- Added `select_validator` for deterministic lab funding (Alice/node 0).

## Updated Roadmap

### Short-Term

1. Systematically resolve compiler warnings and enforce stricter lint gates.
2. Formalize CLI command contracts (JSON schemas and examples).
3. Harden API controls (CORS, auth, and rate limiting baseline).
4. Complete English localization of student-facing lab texts.

### Mid-Term

1. Add argument-level REPL autocomplete and contextual in-shell help.
2. Extend scenario/macro execution to file-based script workflows.
3. Improve network simulation realism with richer delay and partition models.
4. Add ephemeral-key/privacy mode for classroom sessions.

### Long-Term

1. Optimize snapshot + incremental persistence layout for larger histories.
2. Introduce unified observability (event stream/metrics) across CLI and API.
3. Automate consensus benchmarking across broader scenario matrices.
4. LMS/LTI integration and instructor dashboard.
