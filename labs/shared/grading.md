# Değerlendirme kılavuzu

Her laboratuvar 100 puan üzerinden değerlendirilir:

| Bileşen | Puan |
|---------|------|
| Deney adımlarının tamamlanması | 30 |
| Otomatik durum doğrulaması (`lab run` / assertion’lar) | 30 |
| Kavramsal sorular | 25 |
| Sonuçların yorumlanması | 15 |

## Otomatik puanlama

```bash
cargo run --bin sim_cli -- --profile classroom lab run <lab-id> --json
```

`passed: true` ise otomatik kısım tam puan alır. Kısmi başarıda `checks` dizisindeki her assertion eşit ağırlıklıdır.

## Öğrenci teslimi

Önerilen teslim paketi:

1. `lab run ... --json` veya `lab verify ... --json` çıktısı
2. `questions.md` cevapları
3. Kısa gözlem notu (5–10 cümle)

## Akademik dürüstlük

- Beklenen çıktıları ezberlemek yerine gözlemleri açıklayın.
- Private key veya state dosyasını paylaşmayın.
- Başkasının JSON çıktısını teslim etmeyin.
