# HTTP API Kullanım Kılavuzu

Bu belge, `api_server` binary'si ile sunulan HTTP API yüzeyinin kullanımını açıklar.

## 1) API Sunucusunu Başlatma

```bash
cargo run --bin api_server
```

Varsayılan adres:

- `http://0.0.0.0:3000`

Sunucu açılışında:

1. Önce persisted state (`./data/network_state.json`) yüklenmeye çalışılır.
2. State yoksa varsayılan ağ bootstrap edilir (node kurulumu + genesis).

## 2) Endpoint Özeti

### 2.1 Sağlık ve Ağ Durumu

- `GET /health`
- `GET /network/state`

### 2.2 Node ve Zincir

- `GET /nodes`
- `GET /nodes/{id}`
- `GET /nodes/{id}/blockchain`

### 2.3 Mempool

- `GET /mempool`

### 2.4 Yazma Operasyonları

- `POST /transactions`
- `POST /mine`

Yazma endpoint'leri sonrası state diske kaydedilir.

## 3) Örnek İstekler

### 3.1 Sağlık Kontrolü

```bash
curl http://127.0.0.1:3000/health
```

### 3.2 Ağ Özeti

```bash
curl http://127.0.0.1:3000/network/state
```

### 3.3 Node Listesi

```bash
curl http://127.0.0.1:3000/nodes
```

### 3.4 Belirli Node Detayı

```bash
curl http://127.0.0.1:3000/nodes/0
```

### 3.5 Blockchain Görüntüleme

```bash
curl http://127.0.0.1:3000/nodes/0/blockchain
```

### 3.6 Mempool Görüntüleme

```bash
curl http://127.0.0.1:3000/mempool
```

### 3.7 İşlem Oluşturma

```bash
curl -X POST http://127.0.0.1:3000/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "sender_id": 0,
    "recipient_id": 1,
    "amount_coin": 1.25
  }'
```

### 3.8 Tek Adım Blok Üretimi

```bash
curl -X POST http://127.0.0.1:3000/mine
```

## 4) Hata Formatı

API hata durumlarında tutarlı bir JSON formatı döner:

```json
{
  "code": "bad_request",
  "message": "Geçersiz sender_id veya recipient_id"
}
```

Sık görülen hata kodları:

- `bad_request`
- `not_found`
- `internal_error`

## 5) Operasyonel Notlar

- API ve CLI aynı state dosyasını kullanabilir; paralel yazma akışlarında tek aktif yazıcı önerilir.
- Üretim benzeri ortamlar için sonraki adımda:
  - CORS kısıtlaması,
  - authentication/API key,
  - rate-limit
  eklenmesi tavsiye edilir.
