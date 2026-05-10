# Komut Satırı Arayüzü Referansı

Proje artık iki farklı CLI yüzeyi sunar:

1. **Legacy etkileşimli arayüz** (`cargo run`)  
2. **Yeni nesil stateful CLI aracı** (`cargo run --bin sim_cli`)

Bu doküman, ağırlıklı olarak `sim_cli` kullanımını açıklar.

---

## 1) Yeni Nesil CLI (`sim_cli`)

### 1.1 Temel Çalıştırma

```bash
cargo run --bin sim_cli -- <komut>
```

Global bayraklar:

- `--state-path <dosya>`: state dosya yolu (varsayılan: `./data/network_state.json`)
- `--json`: çıktıları JSON formatında üret

Örnek:

```bash
cargo run --bin sim_cli -- --json status
```

### 1.2 Komut Grupları

#### A) `init`

Ağı sıfırdan kurar ve genesis bloğunu üretir.

```bash
cargo run --bin sim_cli -- init --nodes 5 --difficulty 2 --block-time 60
```

Ek bayrak:

- `--force`: mevcut state dosyasının üzerine yaz

#### B) `status`

Ağ özeti (node sayısı, validator, tip, mempool sayısı) verir.

```bash
cargo run --bin sim_cli -- status
```

#### C) `nodes`

- `nodes list`
- `nodes show <id>`

```bash
cargo run --bin sim_cli -- nodes list
cargo run --bin sim_cli -- nodes show 2
```

#### D) `chain`

- `chain tip`
- `chain show --node-id <id> [--limit N]`

```bash
cargo run --bin sim_cli -- chain tip
cargo run --bin sim_cli -- chain show --node-id 0 --limit 5
```

#### E) `tx`

- `tx create --sender-id <id> --recipient-id <id> --amount-coin <deger>`

```bash
cargo run --bin sim_cli -- tx create --sender-id 0 --recipient-id 1 --amount-coin 1.25
```

#### F) `mempool`

- `mempool list`

```bash
cargo run --bin sim_cli -- mempool list
```

#### G) `mine`

- `mine once`

```bash
cargo run --bin sim_cli -- mine once
```

#### H) `persistence`

- `persistence info`
- `persistence save [--path <dosya>]`
- `persistence load --path <dosya>`

```bash
cargo run --bin sim_cli -- persistence info
cargo run --bin sim_cli -- persistence save --path ./data/backup.json
```

### 1.3 REPL Modu

Komut verilmeden çalıştırıldığında REPL açılır:

```bash
cargo run --bin sim_cli --
```

Alternatif:

```bash
cargo run --bin sim_cli -- repl
```

REPL özellikleri:

- kalıcı history (`.sim_cli_history`),
- autocomplete (kök komut + alt komut),
- typo suggestion,
- `help`, `exit`, `quit` yerleşik komutları.

### 1.4 Senaryolar, Alias ve Macro (Phase 3)

#### Senaryolar

```bash
cargo run --bin sim_cli -- scenario list
cargo run --bin sim_cli -- scenario run quickstart
cargo run --bin sim_cli -- scenario run fork-reorg --primary 0 --secondary 1
```

#### Alias

```bash
cargo run --bin sim_cli -- alias add st status
cargo run --bin sim_cli -- alias list
cargo run --bin sim_cli -- alias remove st
```

#### Macro

```bash
cargo run --bin sim_cli -- macro add demo --cmd "status" --cmd "mempool list"
cargo run --bin sim_cli -- macro run demo
cargo run --bin sim_cli -- macro remove demo
```

REPL içinde hızlı çağrı:

```text
!demo
```

---

## 2) Legacy Etkileşimli Arayüz (`cargo run`)

Bu arayüz önceki komut yapısını korur:

- `bakiye <node_id>`
- `transfer <gonderen_id> <alici_id> <miktar>`
- `durum`
- `blockchain <node_id>`
- `mempool`
- `çıkış` (`exit`, `quit`)

Legacy arayüz, geriye dönük uyumluluk ve manuel deneme amaçlı korunmaktadır; yeni operasyonlar için `sim_cli` önerilir.
