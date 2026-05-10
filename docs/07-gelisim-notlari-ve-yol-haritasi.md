# Geliştirme Notları ve Yol Haritası

## Son Dönem Geliştirmeleri (Detaylı)

Bu bölüm, README yapısının modüler hale getirilmesinden sonra çekirdeğe eklenen teknik geliştirmeleri özetler.

### 1) Gelişmiş Mempool Politikası ve Ücret Sistemi

- Mempool kapasitesi sabit `300 MB` olarak sınırlandı.
- İşlem kabulü için minimum fee-rate (sat/kB) eşiği uygulandı.
- Mempool girişlerinde ek metadata tutulmaya başlandı:
  - yaklaşık işlem boyutu,
  - fee miktarı,
  - fee-rate,
  - geliş sırası.
- Blok üretiminde mempool seçimi fee-rate öncelikli hale getirildi.
- RBF-lite davranışı eklendi:
  - aynı outpoint'i harcayan yeni işlem,
  - yalnızca yeterli fee-rate artışı sağlıyorsa mevcut mempool girişini değiştirebilir.
- Zincir senkronizasyonu ve blok yayını sonrası mempool yeniden doğrulama akışı güçlendirildi.

### 2) Fork/Reorg Simülasyonu ve Zincir Seçimi

- Eşit uzunlukta alternatif zincirlerde seçim kuralı geliştirildi:
  1. zincir uzunluğu,
  2. toplam work skoru,
  3. deterministic tie-break (tip hash).
- Ortak ata (common ancestor) bulunarak reorg derinliği hesaplanabilir hale getirildi.
- Ağ seviyesinde doğrudan fork/reorg üretimi için simülasyon akışı eklendi (`simulate_fork_and_reorg`).
- Reorg sonrası node state ve mempool tutarlılığı entegrasyon testleriyle doğrulandı.

### 3) Persistence Faz 1 + Faz 2

#### Faz 1
- Ağ metaverisi, node zincirleri ve cüzdan kimliği diskte saklanır hale getirildi.
- Crash-safe yazma akışı eklendi:
  - geçici dosyaya yaz,
  - `fsync`,
  - atomik yeniden adlandırma.
- Başlangıçta state yükleme ve recovery akışı eklendi.

#### Faz 2
- Şema sürümleme ve migration (`v1 -> v2`) desteği eklendi.
- Node başına UTXO snapshot + checksum eklendi.
- Checksum uyuşmazlığında güvenli fallback:
  - zincirden UTXO setini yeniden inşa et.
- Mempool state persistence ve yeniden yükleme sonrası policy tabanlı revalidation eklendi.

### 4) HTTP API Programı

- Ayrı bir API binary'si eklendi (`api_server`).
- Temel endpoint seti sağlandı:
  - health,
  - network state,
  - node list/detail,
  - blockchain görüntüleme,
  - mempool görüntüleme,
  - transaction oluşturma,
  - tek adım blok üretimi.
- API yazma operasyonları sonrası persistence katmanına state kaydı bağlandı.

### 5) Yeni Nesil CLI (Faz 1-2-3)

#### Faz 1
- `sim_cli` binary'si ile stateful subcommand mimarisi:
  - init/status/nodes/chain/tx/mempool/mine/persistence.
- Global bayraklar:
  - `--state-path`,
  - `--json`.

#### Faz 2
- REPL modu (komut verilmezse otomatik açılış).
- Kalıcı history.
- Komut önerisi ve temel autocomplete.

#### Faz 3
- Senaryo komutları:
  - `scenario list`,
  - `scenario run quickstart`,
  - `scenario run fork-reorg`.
- Alias yönetimi:
  - add/list/remove + REPL genişletme.
- Macro yönetimi:
  - add/list/remove/run + `!macro` kısa çağrısı.
- Namespace-aware autocomplete (kök komut + alt komut düzeyi).

## Güncel Yol Haritası

### Kısa Vadeli

1. Derleme uyarılarının sistematik temizlenmesi (`clippy -D warnings` hedefi).
2. CLI için komut sözleşmelerinin (JSON schema/örnekler) netleştirilmesi.
3. API katmanında temel güvenlik sertleştirmesi (CORS, auth, rate-limit).

### Orta Vadeli

1. REPL içinde arg-level autocomplete ve bağlamsal yardım paneli.
2. Senaryo/macro komutlarının dosya tabanlı script formatına genişletilmesi.
3. Ağ mesajlaşma modelinin daha gerçekçi gecikme/çatallanma davranışlarıyla iyileştirilmesi.

### Uzun Vadeli

1. Snapshot + incremental state formatının performans odaklı optimize edilmesi.
2. API ile CLI arasında ortak gözlemlenebilirlik katmanı (event stream / metrics).
3. Konsensüs deneylerinin farklı parametre kümeleriyle otomatik benchmark edilmesi.
