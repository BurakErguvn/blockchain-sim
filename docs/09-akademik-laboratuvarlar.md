# Akademik Laboratuvar Kullanımı

Bu belge, `labs/` altındaki pedagojik modüllerin sınıf ortamında nasıl kullanılacağını özetler.

## Kurulum

```bash
cargo run --bin sim_cli -- --profile classroom config validate
cargo run --bin sim_cli -- --profile classroom lab list
```

`config/classroom.toml` profili:

- 3 node
- `difficulty = 1`
- kısa blok süresi
- yerel API bind (`127.0.0.1`)
- ayrı state dosyası (`./data/classroom_state.json`)

## Öğretmen akışı

```bash
cargo run --bin sim_cli -- --profile classroom lab show 01-utxo-and-transfers
cargo run --bin sim_cli -- --profile classroom lab run 01-utxo-and-transfers --json
```

`lab run` kurulumu, demo adımlarını ve assertion’ları tek seferde çalıştırır. CI ve ders öncesi smoke test için uygundur.

## Öğrenci akışı

1. İlgili modülün `README.md` dosyasını okuyun.
2. Ortamı kurun:

```bash
cargo run --bin sim_cli -- --profile classroom lab setup 01-utxo-and-transfers
```

3. Elle komutları uygulayın (`tx`, `mempool`, `mine`, `nodes`, `chain`).
4. Soruları (`questions.md`) yanıtlayın.
5. Mümkünse otomatik kontrol çalıştırın:

```bash
cargo run --bin sim_cli -- --profile classroom lab run 01-utxo-and-transfers --json
```

> Not: İmza, double-spend ve fork demoları gibi bayrak gerektiren kontroller için `lab run` kullanın. `lab verify` yalnızca mevcut state assertion’larını kontrol eder ve demo bayraklarını bilmez.

## Değerlendirme

Ortak rubrik: [labs/shared/grading.md](../labs/shared/grading.md)

## Modül listesi

Ayrıntılar için [labs/README.md](../labs/README.md) dosyasına bakın.
