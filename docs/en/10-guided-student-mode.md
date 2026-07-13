# Guided Student Mode

The `sim_cli learn` command group walks students through labs step by step. Students run the blockchain commands themselves; `learn check` only validates the current network state (or runs a safe observation demo).

## Quick start

```bash
cargo run --bin sim_cli -- --profile classroom learn start 01-utxo-and-transfers
cargo run --bin sim_cli -- --profile classroom learn status
cargo run --bin sim_cli -- --profile classroom learn hint
cargo run --bin sim_cli -- --profile classroom learn check
cargo run --bin sim_cli -- --profile classroom learn resume
cargo run --bin sim_cli -- --profile classroom learn reset
```

## Commands

| Command | Purpose |
|---------|---------|
| `learn start <lab-id>` | Deterministic state + new session |
| `learn status` | Progress, attempts, and hint summary |
| `learn hint` | Progressive hint reveal |
| `learn check [--ack TOKEN]` | Validate the current step |
| `learn next [--force]` | Advance (normally automatic after a passing check) |
| `learn resume` | Redisplay the active session |
| `learn reset` | Clear session + state |

## Guided labs available

- `01-utxo-and-transfers`
- `02-signatures-and-integrity`
- `05-forks-and-reorgs`

Other labs remain available through `lab run` automated demos until `guided_steps` are added.

## Session files

Learning progress is stored separately from blockchain state:

```text
data/classroom_state.json
data/learning_sessions/<session-id>.json
data/.sim_cli_learning_active.json
```

Private keys are never printed by learn commands. Session files contain no personal identifiers.

## Feedback model

Failed checks return:

1. Observed
2. Likely cause
3. Next action

## Grading note

Attempt counts do not automatically reduce score. Hint usage is recorded for optional rubric review.
