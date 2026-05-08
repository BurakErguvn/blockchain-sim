# Mutabakat ve Güvenlik

## Mutabakat Yaklaşımı

Simülasyon, çoğunluk temelli bir zincir kabul modeli uygular. Düğümler:

- Gelen blokların yapısal doğruluğunu kontrol eder.
- Zincir bütünlüğünü (önceki hash bağları) doğrular.
- Proof of Work koşulunu denetler.

Zincir senkronizasyonunda daha uzun ve doğrulanmış zincir, yerel zincirin yerine alınabilir.

## Blok Doğrulama Adımları

Bir blok, aşağıdaki kontrollerin tamamı sağlandığında geçerli kabul edilir:

1. Blok indeksi beklenen sırayı izlemelidir.
2. `previous_hash`, yerel son bloğun hash’i ile eşleşmelidir.
3. Blok hash’i yeniden hesaplandığında aynı sonucu vermelidir.
4. Hash, tanımlı zorluk seviyesine uygun olmalıdır.
5. Merkle kökü işlem listesiyle tutarlı olmalıdır.
6. İlk işlem coinbase olmalı; takip eden işlemler coinbase olmamalıdır.
7. Coinbase ödülü, `base_reward + toplam_fee` üst sınırını aşmamalıdır.

## İşlem Güvenliği

İşlem düzeyinde güvenlik aşağıdaki mekanizmalarla sağlanır:

- ECDSA imza doğrulaması
- OutPoint sahiplik eşleştirmesi
- Input tekrar (double reference) engeli
- Mempool outpoint çakışma kontrolü

## Manipülasyon Senaryoları

Simülasyon, aşağıdaki manipülasyon türlerini inceleyebilir:

- Hash manipülasyonu
- Blok içeriği manipülasyonu
- Yerel zincir sapması

Bu senaryolar, doğrulama adımlarının hangi koşullarda manipülasyonu tespit ettiğini gözlemlemek için kullanılır.
