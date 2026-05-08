# Sistem Mimarisi

## Genel Mimarik Düzen

Sistem, modüler bir Rust mimarisi ile aşağıdaki ana bileşenlere ayrılmıştır:

- `node.rs`
- `network.rs`
- `block.rs`
- `transaction.rs`
- `wallet.rs`
- `main.rs`

Bu ayrım, her bileşenin sorumluluğunu netleştirir ve test edilebilirliği artırır.

## Bileşen Sorumlulukları

### Node Katmanı

`Node`, bir katılımcının yerel durumunu taşır:

- Yerel blockchain kopyası
- Yerel UTXO seti
- Wallet durumu
- Geçici mempool
- Validator niteliği

Node aynı zamanda işlem ve blok doğrulama işlevlerini içerir.

### Network Katmanı

`BlockchainNetwork`, düğümler arası koordinasyonu yönetir:

- Düğüm ekleme ve bağlama
- Transaction yayılımı
- Blok yayılımı
- Validator seçimi
- Mempool düzeyi çakışma kontrolü

### Block Katmanı

`Block` bileşeni:

- İşlem listesini taşır
- Merkle kökünü hesaplar
- Hash üretimini yapar
- Proof of Work döngüsünü yürütür

### Transaction Katmanı

`Transaction` modeli:

- Input/Output yapısı
- İşlem kimliği üretimi
- İmza yükü (signing payload) üretimi
- Ücret hesaplama
- Geçerlilik kuralları

### Wallet Katmanı

`Wallet`:

- ECDSA anahtar üretimi
- Adres türetimi
- İmza üretimi ve doğrulama yardımcıları
- Yerel UTXO ve bakiye yönetimi

## Veri Akışı Özeti

1. Düğüm işlem üretir ve imzalar.
2. Ağ, işlemi uygun düğümlere yayar.
3. Validator mempool’dan işlemleri seçer.
4. Block üretimi ve PoW tamamlanır.
5. Blok ağa yayılır, düğümler yerel durumlarını günceller.
