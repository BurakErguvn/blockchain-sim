# Komut Satırı Arayüzü Referansı

Uygulama başlatıldığında etkileşimli bir komut satırı arayüzü sunar.

## Komutlar

### 1) `bakiye <node_id>`

Belirtilen düğümün cüzdan bakiyesini görüntüler.

Örnek:

```text
bakiye 2
```

### 2) `transfer <gonderen_id> <alici_id> <miktar>`

Gönderen düğümden alıcı düğüme coin transferi başlatır.

Örnek:

```text
transfer 0 3 1.25
```

### 3) `durum`

Ağdaki düğümlerin genel durum özetini listeler.

### 4) `blockchain <node_id>`

Belirtilen düğümün yerel zincir özetini gösterir.

### 5) `mempool`

Ağ mempool’unda bekleyen işlemleri listeler.

### 6) `çıkış` (eşanlamlı: `exit`, `quit`)

Simülasyonu sonlandırır.

## Hata Durumları

Arayüz aşağıdaki hataları kullanıcıya bildirir:

- Geçersiz düğüm kimliği
- Eksik parametre
- Sayısal biçim hatası
- Bakiye yetersizliği

Bu hata mesajları, simülasyon akışının gözlemlenebilir ve denetlenebilir olmasını amaçlar.
