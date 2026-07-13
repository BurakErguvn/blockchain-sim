# Lab 05 — Fork ve Reorg

## Öğrenme hedefleri

- Fork oluşumunu açıklamak
- Canonical chain seçimini gözlemlemek
- Reorg derinliğini ölçmek

## Deney

```bash
cargo run --bin sim_cli -- --profile classroom lab run 05-forks-and-reorgs --json
```

Elle:

```bash
cargo run --bin sim_cli -- --profile classroom lab setup 05-forks-and-reorgs
cargo run --bin sim_cli -- --profile classroom mine once
cargo run --bin sim_cli -- --profile classroom scenario run fork-reorg --json
cargo run --bin sim_cli -- --profile classroom --json chain tip
cargo run --bin sim_cli -- --profile classroom --json nodes list
```

Beklenen: `reorg_depth = 1` ve tüm node tip’lerinin aynı hash’e gelmesi.
