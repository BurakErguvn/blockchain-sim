# Giriş ve Kapsam

## Amaç

Bu proje, blockchain sistemlerinin temel çalışma prensiplerini açıklamak üzere geliştirilmiş bir Rust simülasyonudur. Uygulama, dağıtık düğüm yapısı, blok üretimi, doğrulama, işlem akışı ve UTXO tabanlı bakiye yönetimini öğretici bir çerçevede bir araya getirir.

## Kapsam Sınırları

Bu çalışma, üretim seviyesinde bir kripto para ağı yerine akademik amaçlı bir model sunar. Bu nedenle:

- Ağ katmanı gerçek internet topolojisi yerine simülasyon modeliyle temsil edilir.
- Konsensüs mekanizması kavramsal doğruluk odaklıdır.
- Komut satırı etkileşimi, eğitim ve gözlem için tasarlanmıştır.

## Temel Kavramlar

- **Node (Düğüm):** Zincirin bir kopyasını tutan ve ağla iletişim kuran katılımcı.
- **Validator:** Geçici olarak blok üretme yetkisi verilen düğüm.
- **Block:** İşlem listesi, önceki blok özeti, nonce ve zaman damgası gibi alanları içeren veri yapısı.
- **UTXO:** Harcanmamış işlem çıktısı; bakiye hesaplamasının temel birimi.
- **Proof of Work:** Blok hash’inin belirli bir zorluk koşulunu sağlamasını zorunlu kılan mekanizma.

## Hedef Kullanım Senaryoları

Bu proje aşağıdaki amaçlar için uygundur:

- Blockchain kavramlarını ders/ödev kapsamında incelemek
- Rust ile veri yapısı ve doğrulama akışları üzerine çalışmak
- UTXO, imza doğrulama ve mutabakat ilişkisini deneysel olarak gözlemlemek
