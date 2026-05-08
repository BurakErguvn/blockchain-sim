# İşlem ve UTXO Modeli

## İşlem Modeli

Her işlem aşağıdaki temel yapılardan oluşur:

- **Input (`TxInput`)**: Harcanacak çıktıyı (`OutPoint`) gösterir ve imza verisini taşır.
- **Output (`TxOutput`)**: Miktar ve alıcı adres bilgisi içerir.
- **Transaction ID**: İşlem içeriğinden türetilen özet değerdir.

## OutPoint Tabanlı Kimlik

UTXO kimliği, string birleştirme yerine tip güvenli bir modelle temsil edilir:

- `OutPoint { txid, vout }`

Bu yaklaşımın sonuçları:

- Kimlik ayrıştırma hataları azaltılır.
- Veri yapısı düzeyinde tip güvenliği artar.
- Lookup işlemleri `HashMap` ile daha deterministik hale gelir.

## UTXO Set Yapısı

Node ve wallet katmanında UTXO verisi `HashMap<OutPoint, UTXO>` olarak tutulur. Böylece:

- Harcanacak çıktının bulunması doğrudan anahtar üzerinden yapılır.
- Durum güncellemeleri sırasında çakışma ve tekrar kontrolü sadeleşir.

## İmza ve Sahiplik Doğrulaması

Bir işlemin geçerli sayılabilmesi için aşağıdaki koşullar birlikte sağlanır:

1. Input’ta referans verilen UTXO mevcut olmalıdır.
2. Input’taki public key’den türetilen adres, UTXO alıcı adresi ile eşleşmelidir.
3. Input imzası, ilgili signing payload üzerinde doğrulanmalıdır.
4. Aynı OutPoint aynı işlem içinde tekrar kullanılamaz.
5. Toplam output, toplam input’tan büyük olamaz.

## Ücret (Fee) Modeli

Coinbase dışındaki işlemler için ücret:

`fee = total_input - total_output`

Bu değer blok üretimi sırasında toplanır ve coinbase ödülüne eklenir. Üretilecek coinbase değeri, temel madencilik ödülü ile blok içindeki toplam işlem ücretinin toplamını aşamaz.
