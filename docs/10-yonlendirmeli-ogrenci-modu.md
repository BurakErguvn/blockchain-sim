# Yönlendirmeli Öğrenci Modu

`sim_cli learn` komut grubu, laboratuvarları adım adım yönlendirir. Öğrenci komutları kendisi çalıştırır; `learn check` yalnızca mevcut ağı doğrular (veya güvenli bir gözlem demosu çalıştırır).

## Hızlı başlangıç

```bash
cargo run --bin sim_cli -- --profile classroom learn start 01-utxo-and-transfers
cargo run --bin sim_cli -- --profile classroom learn status
cargo run --bin sim_cli -- --profile classroom learn hint
cargo run --bin sim_cli -- --profile classroom learn check
cargo run --bin sim_cli -- --profile classroom learn resume
cargo run --bin sim_cli -- --profile classroom learn reset
```

## Komutlar

| Komut | Amaç |
|-------|------|
| `learn start <lab-id>` | Deterministik state + yeni session |
| `learn status` | İlerleme, deneme ve ipucu özeti |
| `learn hint` | Kademeli ipucu |
| `learn check [--ack TOKEN]` | Mevcut adımı doğrula |
| `learn next [--force]` | Sonraki adıma geç (normalde check sonrası otomatik) |
| `learn resume` | Aktif oturumu yeniden göster |
| `learn reset` | Session + state temizliği |

## Desteklenen yönlendirmeli laboratuvarlar

- `01-utxo-and-transfers`
- `02-signatures-and-integrity`
- `05-forks-and-reorgs`

Diğer laboratuvarlar `lab run` ile otomatik demo olarak kullanılmaya devam eder; `guided_steps` eklendikçe `learn` kapsamına alınabilir.

## Session dosyaları

Blockchain state ile öğrenme ilerlemesi ayrıdır:

```text
data/classroom_state.json
data/learning_sessions/<session-id>.json
data/.sim_cli_learning_active.json
```

Private key’ler öğrenme çıktılarında gösterilmez. Session dosyaları kişisel bilgi içermez.

## Pedagojik geri bildirim

Başarısız `learn check` üç parça döner:

1. Observed — ne görüldü
2. Likely cause — olası kavramsal neden
3. Next action — öğrenci neyi denemeli

## Değerlendirme notu

Deneme sayısı doğrudan puan düşürmez. İpucu kullanımı oturumda kaydedilir ve rubrikte isteğe bağlı değerlendirilebilir.
