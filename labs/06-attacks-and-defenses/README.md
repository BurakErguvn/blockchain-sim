# Lab 06 — Saldırılar ve Savunmalar

## Öğrenme hedefleri

- Double-spend denemesini gözlemlemek
- İmza ve zincir manipülasyonunun hangi katmanda engellendiğini eşleştirmek
- Simülasyonun güvenlik sınırlarını tartışmak

## Deney

```bash
cargo run --bin sim_cli -- --profile classroom lab run 06-attacks-and-defenses --json
```

Demo adımları:

1. Aynı UTXO ile ikinci harcama reddi
2. İmzalı işlem manipülasyonu reddi
3. Zincir manipülasyonu sonrası tutarlılık kontrolü

## Güvenlik notu

State dosyasında private key saklanabilir. Paylaşımlı lab bilgisayarlarında ders sonunda state’i silin.
