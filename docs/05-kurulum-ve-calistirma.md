# Kurulum ve Çalıştırma

## Ön Koşullar

- Rust (stable)
- Cargo

Kurulum doğrulaması için:

```bash
rustc --version
cargo --version
```

## Projeyi Çalıştırma

1. Depoyu klonlayın.
2. Proje dizinine geçin.
3. Uygulamayı başlatın:

```bash
cargo run
```

## Test ve Derleme Doğrulama

### Birim ve entegrasyon testleri

```bash
cargo test
```

### Derleme kontrolü

```bash
cargo check
```

## Çalışma Zamanı Notları

- Uygulama, komut satırı arayüzü üzerinden etkileşimlidir.
- Ağ durumu ve blok üretimi, simülasyon parametrelerine göre değişir.
- PoW zorluğu ve blok süresi kod içinde ayarlanabilir.
