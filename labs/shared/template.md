# Yeni laboratuvar şablonu

Bu şablonu kopyalayarak yeni bir modül ekleyin.

## Dizin

```text
labs/NN-short-name/
├── README.md
├── lab.toml
├── questions.md
├── rubric.md
├── instructor-notes.md
└── expected-output.json
```

## `lab.toml` iskeleti

```toml
id = "NN-short-name"
title = "Title"
difficulty = "beginner"
estimated_minutes = 25
objectives = ["Objective 1"]
aliases = ["Alice", "Bob", "Carol"]

[setup]
nodes = 3
difficulty = 1
block_time_seconds = 1
mine_genesis = true

[[steps]]
action = "mine_block"

[[assertions]]
type = "chain_height_min"
expected = 1
```

## Desteklenen adımlar

- `create_transaction`
- `mine_block`
- `set_difficulty`
- `simulate_fork_reorg`
- `demonstrate_signature_tamper`
- `demonstrate_double_spend`
- `demonstrate_chain_tamper`

## Desteklenen assertion’lar

- `node_count`
- `balance_equals` / `balance_min`
- `chain_height_min`
- `mempool_count_min` / `mempool_count_equals`
- `tip_hash_present`
- `difficulty_equals`
- `canonical_tips_match`
- `reorg_depth_equals`
- `demo_flag_true`
