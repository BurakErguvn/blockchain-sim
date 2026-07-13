# Akademik Laboratuvar Paketi

Bu dizin, blockchain simülasyonunu ders/laboratuvar ortamında kullanmak için hazırlanmış pedagojik modülleri içerir.

## Hızlı başlangıç

```bash
# Sınıf profili ile CLI
cargo run --bin sim_cli -- --profile classroom lab list

# Otomatik demo + doğrulama (öğretmen / CI)
cargo run --bin sim_cli -- --profile classroom lab run 01-utxo-and-transfers --json

# Öğrenci kurulumu
cargo run --bin sim_cli -- --profile classroom lab setup 01-utxo-and-transfers
```

## Modüller

| ID | Başlık | Seviye | Süre |
|----|--------|--------|------|
| `01-utxo-and-transfers` | UTXO ve işlemler | beginner | ~25 dk |
| `02-signatures-and-integrity` | İmzalar ve bütünlük | beginner | ~25 dk |
| `03-mempool-and-fees` | Mempool ve ücretler | intermediate | ~30 dk |
| `04-proof-of-work` | Proof of Work | intermediate | ~30 dk |
| `05-forks-and-reorgs` | Fork ve reorg | intermediate | ~35 dk |
| `06-attacks-and-defenses` | Saldırılar ve savunmalar | advanced | ~40 dk |

## Her modülde neler var?

- `README.md` — öğrenci yönergesi
- `lab.toml` — kurulum, adımlar ve otomatik assertion’lar
- `questions.md` — kavramsal sorular
- `rubric.md` — puanlama rubriği
- `instructor-notes.md` — öğretmen notları
- `expected-output.json` — örnek başarılı çıktı şeması

## Ortak materyaller

- [glossary.md](shared/glossary.md)
- [troubleshooting.md](shared/troubleshooting.md)
- [grading.md](shared/grading.md)
- [template.md](shared/template.md)

## Node takma adları

Sınıf profilinde varsayılan node eşlemesi:

- `0` → Alice
- `1` → Bob
- `2` → Carol

## Önemli sınırlar

Bu bir eğitim simülasyonudur:

- Gerçek internet topolojisi yoktur.
- Validator seçimi basitleştirilmiştir.
- Private key’ler state dosyasında saklanabilir; paylaşımlı ortamlarda dikkatli olun.
