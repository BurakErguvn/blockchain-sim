# Sık karşılaşılan sorunlar

## `state already exists`

`init` veya `lab setup` mevcut state üzerine yazmaz. `--force` ile sıfırlayın:

```bash
cargo run --bin sim_cli -- --profile classroom init --force
```

## Genesis sonrası bakiyeler sıfır görünüyor

Yalnızca genesis bloğunu üreten validator ödül alır. Önce `nodes list` ile fonlanmış node’u bulun; laboratuvarlarda gönderen genelde node `0` kabul edilir, ancak fonlanmış node farklıysa onu kullanın.

Laboratuvar motoru (`lab run`) genesis sonrası işlemleri fonlanmış node üzerinden kurduğu için otomatik demoda bu sorun oluşmaz. Elle çalışırken:

```bash
cargo run --bin sim_cli -- --profile classroom --json nodes list
```

## `tx create` reddediliyor

Kontrol listesi:

1. Gönderenin bakiyesi yeterli mi?
2. `amount_coin` sıfırdan büyük mü?
3. Mempool’da aynı UTXO’yu harcayan işlem var mı?
4. Fee çok düşük mü?

## Fork lab’ında reorg oluşmuyor

`lab run 05-forks-and-reorgs` kullanın. Elle deniyorsanız önce en az bir blok daha üretin, ardından senaryoyu çalıştırın:

```bash
cargo run --bin sim_cli -- --profile classroom mine once
cargo run --bin sim_cli -- --profile classroom scenario run fork-reorg --json
```

## Classroom profili yüklenmiyor

Komutu proje kökünden çalıştırın ve `config/classroom.toml` dosyasının varlığını doğrulayın:

```bash
cargo run --bin sim_cli -- --profile classroom config paths
```
