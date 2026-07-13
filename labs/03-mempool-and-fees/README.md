# Lab 03 — Mempool ve Ücretler

## Öğrenme hedefleri

- Mempool’un rolünü açıklamak
- Fee ile fee-rate farkını ayırt etmek
- Yüksek fee-rate’li işlemin önceliğini gözlemlemek

## Deney

```bash
cargo run --bin sim_cli -- --profile classroom lab run 03-mempool-and-fees --json
cargo run --bin sim_cli -- --profile classroom --json mempool list
```

Elle denemek için:

```bash
cargo run --bin sim_cli -- --profile classroom lab setup 03-mempool-and-fees
cargo run --bin sim_cli -- --profile classroom tx create --sender-id 0 --recipient-id 1 --amount-coin 20
cargo run --bin sim_cli -- --profile classroom mine once
cargo run --bin sim_cli -- --profile classroom tx create --sender-id 0 --recipient-id 2 --amount-coin 1 --fee-satoshi 1000
cargo run --bin sim_cli -- --profile classroom tx create --sender-id 1 --recipient-id 2 --amount-coin 1 --fee-satoshi 50000
cargo run --bin sim_cli -- --profile classroom --json mempool list
```

Fee-rate değerlerini karşılaştırın; ardından `mine once` ile hangi işlemin seçildiğini tartışın.
