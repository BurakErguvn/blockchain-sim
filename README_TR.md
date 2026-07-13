# Blockchain Simülasyonu

Merhaba, hoş geldin. Bu proje; blok üretimi, işlem doğrulama, UTXO takibi, mempool politikaları, persistence, API ve gelişmiş CLI deneyimini tek bir öğrenme ve deneme ortamında bir araya getiren topluluk dostu bir blockchain simülasyonudur.

Bu README dosyasını hızlı bir başlangıç noktası gibi düşünebilirsin: nereden başlaman gerektiğini gösterir, seni doğru dokümana yönlendirir ve projeye katkı vermeyi kolaylaştırır. Daha derin teknik detaylar `docs/` dizinindeki modüler belgelerde tutulur.

İngilizce dokümantasyon girişini kullanmak istersen: [README (EN)](README.md)

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
- [HTTP API Kullanım Kılavuzu](docs/08-http-api-kullanimi.md)
- [Akademik Laboratuvarlar](docs/09-akademik-laboratuvarlar.md)
- [Yönlendirmeli Öğrenci Modu](docs/10-yonlendirmeli-ogrenci-modu.md)
- [Ratatui Öğrenci Arayüzü](docs/11-ratatui-arayuzu.md)

## Kısa Çalıştırma Notu

```bash
cargo run
```

Ek çalıştırma örnekleri:

```bash
cargo run --bin sim_cli -- status
cargo run --bin api_server
cargo run --bin sim_cli -- --profile classroom lab list
cargo run --bin sim_cli -- --profile classroom lab run 01-utxo-and-transfers --json
cargo run --bin sim_cli -- --profile classroom tui
```

Akademik laboratuvar paketi: [labs/README.md](labs/README.md)