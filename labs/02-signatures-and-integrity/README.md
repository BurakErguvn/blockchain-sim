# Lab 02 — İmzalar ve Bütünlük

## Öğrenme hedefleri

- İşlemlerin neden imzalandığını açıklamak
- Hash ile dijital imzanın farkını ayırt etmek
- Değiştirilmiş bir işlemin doğrulamada reddedildiğini gözlemlemek

## Kısa teori

İmza, “bu girdileri harcamaya yetkim var” iddiasını kriptografik olarak kanıtlar. Çıktı miktarı değiştirilirse eski imza artık yeni içeriği kapsamaz; doğrulama başarısız olur.

## Başlangıç

```bash
cargo run --bin sim_cli -- --profile classroom lab setup 02-signatures-and-integrity
```

## Deney

Elle bir geçerli işlem oluşturup mempool’a ekleyin, ardından otomatik demo ile manipülasyonu görün:

```bash
cargo run --bin sim_cli -- --profile classroom tx create --sender-id 0 --recipient-id 1 --amount-coin 5
cargo run --bin sim_cli -- --profile classroom lab run 02-signatures-and-integrity --json
```

Demo, geçerli bir işlemi yerelde bozup `verify_transaction` ile reddi gösterir.

## Gözlem odakları

- Değişen alan: output amount
- Yeniden hesaplanan alan: tx id / hash
- Değişmeyen ama artık geçersiz olan: eski imza
