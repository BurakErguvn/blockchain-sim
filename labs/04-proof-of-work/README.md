# Lab 04 — Proof of Work

## Öğrenme hedefleri

- Nonce, difficulty ve block hash ilişkisini kurmak
- Difficulty artışının maliyet etkisini tartışmak
- Üretilen bloğun hedefi sağladığını doğrulamak

## Deney

```bash
cargo run --bin sim_cli -- --profile classroom lab run 04-proof-of-work --json
cargo run --bin sim_cli -- --profile classroom --json chain tip
cargo run --bin sim_cli -- --profile classroom --json status
```

İsteğe bağlı karşılaştırma:

```bash
cargo run --bin sim_cli -- --profile classroom init --force --difficulty 1
cargo run --bin sim_cli -- --profile classroom mine once
# ardından daha yüksek difficulty ile yeniden deneyin
cargo run --bin sim_cli -- --profile classroom init --force --difficulty 3
cargo run --bin sim_cli -- --profile classroom mine once
```

Süreleri not edin; sınıf bilgisayarlarında `difficulty=1` tercih edin.
