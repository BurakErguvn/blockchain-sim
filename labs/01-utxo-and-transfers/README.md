# Lab 01 — UTXO ve İşlemler

## Öğrenme hedefleri

- UTXO modelini açıklamak
- Girdi, çıktı ve para üstü (change) ilişkisini görmek
- Bakiyenin onaydan önce/sonra nasıl değiştiğini gözlemlemek

## Ön koşullar

- Rust toolchain kurulu
- Proje kökünden çalıştırma

## Kısa teori

Bakiye tek bir sayı değildir. Cüzdan, kendisine ait harcanmamış çıktıların (UTXO) toplamını tutar. Bir transfer:

1. bir veya daha fazla UTXO harcar,
2. alıcıya yeni bir çıktı üretir,
3. gerekirse gönderene para üstü döndürür,
4. ücret için bir fark bırakır.

## Başlangıç

```bash
cargo run --bin sim_cli -- --profile classroom lab setup 01-utxo-and-transfers
cargo run --bin sim_cli -- --profile classroom --json nodes list
```

Takma adlar: `0=Alice`, `1=Bob`, `2=Carol`. Genesis sonrası Alice fonlanmıştır.

## Deney adımları

1. Alice ve Bob bakiyelerini kaydedin.
2. Transfer oluşturun:

```bash
cargo run --bin sim_cli -- --profile classroom tx create --sender-id 0 --recipient-id 1 --amount-coin 10
cargo run --bin sim_cli -- --profile classroom --json mempool list
```

3. Onaydan önce bakiyeleri tekrar kontrol edin.
4. Blok üretin:

```bash
cargo run --bin sim_cli -- --profile classroom mine once
cargo run --bin sim_cli -- --profile classroom --json nodes list
```

5. Zincir ucunu inceleyin:

```bash
cargo run --bin sim_cli -- --profile classroom --json chain tip
```

## Otomatik demo

```bash
cargo run --bin sim_cli -- --profile classroom lab run 01-utxo-and-transfers --json
```

## Gözlem soruları

`questions.md` dosyasını doldurun.
