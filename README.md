# Blockchain Simülasyonu

Bu dosya, proje dokümantasyonu için yönlendirme indeksidir. Ayrıntılı içerik, `docs/` dizinindeki modüler belgelerde sunulmaktadır.

## Dokümantasyon Girişi

- [Dokümantasyon Dizini](docs/README.md)

## Hızlı Erişim

- [Giriş ve Kapsam](docs/01-giris-ve-kapsam.md)
- [Sistem Mimarisi](docs/02-sistem-mimarisi.md)
- [İşlem ve UTXO Modeli](docs/03-islem-ve-utxo-modeli.md)
- [Mutabakat ve Güvenlik](docs/04-mutabakat-ve-guvenlik.md)
- [Kurulum ve Çalıştırma](docs/05-kurulum-ve-calistirma.md)
- [Komut Satırı Arayüzü Referansı](docs/06-komut-satiri-arayuzu.md)
- [Geliştirme Notları ve Yol Haritası](docs/07-gelisim-notlari-ve-yol-haritasi.md)

## Son Güncellemeler (README Yenilemesinden Sonra)

- **Gelişmiş mempool politikası** eklendi: sabit `300 MB` kapasite, minimum fee-rate kabul kuralı, fee-rate öncelikli blok seçimi, RBF-lite replacement.
- **Fork/Reorg simülasyonu** eklendi: eşit uzunlukta zincirlerde work-score karşılaştırması, deterministic tie-break, reorg derinliği ölçümü ve ağ senaryo testi.
- **Persistence Faz 1 ve Faz 2** tamamlandı: disk state kaydı/yükleme, startup recovery, schema migration (`v1 -> v2`), UTXO snapshot + checksum, mempool persistence ve yeniden doğrulama.
- **HTTP API programı** eklendi (`cargo run --bin api_server`): health, network state, node/blockchain, mempool, transaction ve mine endpointleri.
- **Yeni nesil CLI aracı** eklendi (`cargo run --bin sim_cli`): stateful komutlar, REPL, history, autocomplete, typo suggestion, scenario/alias/macro yönetimi.

## Kısa Çalıştırma Notu

```bash
cargo run
```

Ek çalıştırma örnekleri:

```bash
cargo run --bin sim_cli -- status
cargo run --bin api_server
```
