# Lab 01 — Öğretmen notları

## Demo akışı (5–7 dk)

1. `lab setup` ile temiz state
2. `nodes list` ile Alice bakiyesini göster
3. Transfer + mempool
4. Mine + bakiye karşılaştırması
5. İsterseniz `lab run --json` ile assertion’ları gösterin

## Beklenen sonuç

- Bob bakiyesi ≥ 10 coin
- Zincir yüksekliği ≥ 2
- Tip hash mevcut

## Yaygın hatalar

- Fonlanmamış node’dan göndermeye çalışmak
- Onay öncesi bakiye değişimini “bug” sanmak

## Basitleştirme uyarısı

Tek süreçli simülasyonda ağ gecikmesi yoktur; mempool gözlemi anlıktır.
