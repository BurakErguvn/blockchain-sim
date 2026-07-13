# Ratatui Öğrenci Arayüzü

Klasik CLI komutları otomasyon ve JSON tüketimi için korunur. Etkileşimli ders kullanımı için Ratatui tabanlı tam ekran arayüz önerilir.

## Başlatma

```bash
cargo run --bin sim_cli -- --profile classroom tui
```

Özel state veya laboratuvar dizini:

```bash
cargo run --bin sim_cli -- \
  --profile classroom \
  --state-path ./data/student-01.json \
  tui --labs-root ./labs
```

## Ekran düzeni

- **Labs**: Laboratuvar seçimi; yeşil nokta yönlendirmeli modu destekler.
- **Tasks**: Tamamlanan, aktif ve gelecek görevler.
- **Task workspace**: Yönerge, kavram, komut önerisi ve tartışma sorusu.
- **Chain movement**: Son bloklar, zincir ucu ve blok hareketi.
- **Network**: Node bakiyeleri, validator ve zincir yükseklikleri.
- **Mempool**: Bekleyen işlemler.
- **Activity**: İşlem, blok, kontrol, ipucu ve hata olayları.

## Klavye

| Tuş | Eylem |
|-----|-------|
| `Tab`, `←`, `→` | Panel odağını değiştir |
| `↑`, `↓`, `j`, `k` | Seçimi hareket ettir |
| `Enter`, `a` | Lab başlat veya aktif görevin önerilen eylemini uygula |
| `c` | Aktif görevi kontrol et |
| `h` | Sıradaki ipucunu göster |
| `m` | Bir blok üret |
| `r` | State dosyasını yeniden yükle |
| `?` | Yardım |
| `q`, `Esc`, `Ctrl-C` | Güvenli çıkış |

## Tasarım kararları

Mevcut düz metin çıktıların temel sorunları:

1. Ağ, görev ve zincir bilgileri farklı komutlara dağılmıştı.
2. Hash ve state alanları görsel hiyerarşi olmadan listeleniyordu.
3. Öğrenci hangi görevin aktif olduğunu sürekli `learn status` ile kontrol etmek zorundaydı.
4. Zincir ve mempool değişimleri ardışık çıktılardan elle karşılaştırılıyordu.

Yeni arayüz bu bilgileri tek ekranda toplar. Klasik komutların davranışını değiştirmez; CI, script ve erişilebilir düz metin kullanımını korur.

## Görev güvenliği

- Görevler sırayla tamamlanır.
- Gelecek görevler incelenebilir fakat uygulanamaz.
- `Enter` yalnızca manifestteki önerilen güvenli eylemi çalıştırır.
- Private key arayüzde gösterilmez.
- Terminal, panic veya normal çıkışta raw mode'dan çıkarılmaya çalışılır.

## Sonraki arayüz geliştirme planı

1. Terminal boyutu küçükken kompakt görünüm.
2. Mouse desteği ve panel yeniden boyutlandırma.
3. Fork dalları için dikey graph görünümü.
4. Türkçe/İngilizce UI metin profili.
5. Tema dosyası ve yüksek kontrast seçeneği.
6. Event stream üzerinden API sunucusuna canlı bağlanma.
