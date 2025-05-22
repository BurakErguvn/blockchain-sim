# Blockchain Simülasyonu

Bu proje, bir blockchain ağının temel işleyişini simüle eden Rust tabanlı bir uygulamadır. Gerçek bir blockchain sisteminin temel özelliklerini ve konsept kanıtlamasını (proof of concept) göstermek amacıyla geliştirilmiştir.

> [English Documentation - README_EN.md](README_EN.md)

## Özellikler

### Node Yapısı

- Her node bir blockchain kopyası tutar
- Validatorlar (düğüm doğrulayıcıları) yeni blokları oluşturabilir
- Nodelar arasında bağlantılar ve iletişim vardır
- Her node'un benzersiz bir kimliği (ID) bulunur
- Nodelar kendileriyle bağlantı kuramazlar
- Her node'un kendi cüzdanı ve benzersiz kripto para adresi vardır

### Wallet (Cüzdan) Yapısı

- Özel-genel anahtar çifti (ECDSA) kullanarak güvenli işlemler yapar
- Özel anahtarlar, 256-bit rastgele sayılar kullanılarak oluşturulur
- Genel anahtarlardan Base58 formatında Bitcoin benzeri adresler üretir
- İşlem imzalama ve doğrulama fonksiyonları içerir
- UTXO (Harcanmamış İşlem Çıktıları) modelini kullanarak bakiye yönetimi yapar

### Block Yapısı

- Index: Blokun zincirdeki sıra numarası
- Timestamp: Bloğun oluşturulduğu zaman damgası
- Data: Blok içinde saklanan veriler (transactions)
- Previous Hash: Önceki bloğun hash değeri
- Hash: Mevcut bloğun SHA-256 ile oluşturulmuş hash değeri
- Nonce: Proof of Work algoritması için kullanılan sayaç

### Blockchain Özellikleri

- Genesis Bloğu: Zincirin ilk bloğu
- Immutability (Değiştirilemezlik): Blok verileri değiştirilemez, değişirse tespit edilir
- Consensus (Uzlaşma): Çoğunluk kuralı ile validasyon
- Proof of Work: Madencilik işlemi için gereken zorluğu simüle eder
- Distributed Ledger: Her node tüm blockchain'in bir kopyasını tutar
- Süreli Validator Yetkisi: Validatorlar sadece bir blok oluşturduktan sonra yetkileri kaldırılır
- Madencilik Sonucu Hash Dağıtımı: Nodelara işlem hash'i değil, Proof of Work sonucu oluşan hash dağıtılır

### Güvenlik Özellikleri

- SHA-256 hash algoritması kullanımı
- Blok doğrulama mekanizması
- Manipülasyon tespiti ve düzeltme sistemi
- Çoğunluk tabanlı konsensüs mekanizması
- Tek validatorda güç yoğunlaşmasını önleme sistemi
- ECDSA dijital imzaları ile işlem doğrulama

## Nasıl Çalışır?

1. **Ağ Oluşturma**:

   - Çeşitli nodelar oluşturulur ve birbirine bağlanır (kendileriyle değil)
   - Her node bir cüzdan (özel-genel anahtar çifti) ve adres oluşturur
   - Başlangıçta her node bir Genesis bloğu içerir

2. **Validator Seçimi**:

   - Rastgele bir node validator olarak seçilir
   - Sadece validatorlar yeni blok oluşturabilir
   - Her validator sadece bir blok oluşturabilir, sonra yetkisi alınır

3. **İşlem Oluşturma ve Madencilik**:

   - Yeni bir işlem (transaction) kaynaktan hedefe oluşturulur
   - İşlem, gönderen tarafından dijital olarak imzalanır ve doğrulanır
   - İşlemler UTXO (Harcanmamış İşlem Çıktıları) modeli kullanılarak işlenir
   - Validator bu işlemi alır ve işler (SHA-256 hash'ini oluşturur)
   - Proof of Work algoritması ile yeni bir blok oluşturulur (belirli sayıda öncü sıfır)
   - Blok madenciliği sonucu oluşan hash değeri (nonce'lu hash) tüm ağa dağıtılır
   - Yeni blok ağdaki tüm nodelara yayınlanır
   - Validator'ın yetkisi kaldırılır

4. **Güvenlik ve Doğrulama**:

   - Nodelar blockchain'in bütünlüğünü sürekli kontrol eder
   - Manipülasyon girişimleri tespit edilir
   - Bozulmuş blockchain'ler, çoğunluk kuralı ile düzeltilir

5. **Konsensüs Algoritması**:
   - Ağdaki nodeların çoğunluğu geçerli zinciri belirler
   - Bir node manipüle edildiğinde, diğer nodelar düzeltici önlemler alır

## Simülasyon Senaryoları

Simülasyon şu senaryoları içerir:

1. **Normal İşlem Akışı**:

   - Validator seçilir ve yeni bir işlem (transaction) ekler
   - İşlem dijital olarak imzalanır ve doğrulanır
   - Blok madenciliği yapılır ve zincire eklenir
   - Validator'ın yetkisi kaldırılır

2. **Hash Manipülasyonu**:

   - Normal bir node hash'i değiştirmeye çalışır
   - Konsensüs mekanizması bunu tespit eder ve reddeder

3. **Blockchain Manipülasyonu**:
   - Bir node blockchain verilerini değiştirmeye çalışır
   - Veriyi değiştirdikten sonra PoW kurallarına uygun olarak yeni nonce ve hash hesaplar
   - Diğer nodelar manipülasyonu tespit eder (veriler değiştiği halde PoW kuralları sağlanmış olsa bile)
   - Çoğunluk konsensüsü ile manipülasyon engellenir ve zincir düzeltilir

## Teknik Detaylar

### Kullanılan Teknolojiler

- **Programlama Dili**: Rust
- **Hash Algoritması**: SHA-256 (sha2 crate)
- **Rastgele Sayı Üreteci**: rand crate
- **Kriptografi**: secp256k1 (ECDSA imzalama)
- **Adres Kodlama**: bs58 (Base58 kodlama)

### Proje Yapısı

- **src/main.rs**: Ana simülasyon akışı ve test senaryoları
- **src/node.rs**: Node yapısı ve ilgili implementasyonlar
- **src/block.rs**: Block yapısı ve ilgili fonksiyonlar
- **src/network.rs**: BlockchainNetwork yapısı ve ilgili fonksiyonlar
- **src/wallet.rs**: Cüzdan yapısı, anahtar üretimi, imzalama fonksiyonları ve UTXO yönetimi
- **src/transaction.rs**: İşlem yapısı, UTXO modeli ve işlem doğrulama fonksiyonları
- **LICENSE**: MIT lisansı (Copyright 2024 Burak Ergüven)
- **README.md**: Proje dokümantasyonu

### İçerdiği Özellikler

- **Decentralized (Merkezi Olmayan)**: Nodelar arasında dağıtılmış yapı
- **Transparent (Şeffaf)**: Tüm nodelar blockchain'i görebilir
- **Secure (Güvenli)**: SHA-256 hash ve ECDSA imzalama
- **Immutable (Değiştirilemez)**: Değişiklikler tespit edilir ve düzeltilir
- **Democratic (Demokratik)**: Hiçbir node sürekli kontrol sahibi olamaz
- **Cryptographic Identity (Kriptografik Kimlik)**: Her node benzersiz bir adrese sahip
- **UTXO-Based (UTXO Tabanlı)**: Bitcoin'e benzer şekilde Harcanmamış İşlem Çıktıları modeli kullanılır
- **Transaction Verification (işlem Doğrulama)**: UTXO'ların varlığı ve sahipliği kontrol edilir

## Nasıl Çalıştırılır?

1. Rust ve Cargo'nun yüklü olduğundan emin olun
2. Projeyi klonlayın
3. Terminal'de proje dizinine gidin
4. Aşağıdaki komutu çalıştırın:

```bash
cargo run
```

## Gelecek Geliştirmeler

- Akıllı sözleşme desteği
- Daha sofistike bir P2P ağ simülasyonu

## Son Güncellemeler

### Kullanıcı Arayüzü ve Çıktı İyileştirmeleri (En Son Güncelleme)

- **Komut Arayüzü**: Kullanıcının blockchain ile etkileşime geçebileceği komut arayüzü eklendi
- **Blok Bilgilerinin Ayrı Terminalde Gösterilmesi**: Blok bilgileri ayrı bir terminalde görüntülenerek kullanıcı deneyimi iyileştirildi
- **Gereksiz Mesajların Kaldırılması**: Geri sayım ve gereksiz bildirim mesajları kaldırılarak terminal çıktısı daha temiz hale getirildi
- **Madenci Bilgilerinin Doğru Gösterilmesi**: Blok oluşturan madenci ve yeni seçilen madenci bilgileri doğru şekilde gösteriliyor
- **Çıkış Komutu İyileştirmesi**: Simülasyonun tüm threadleri düzgün şekilde sonlandırması sağlandı

### UTXO Tabanlı İşlem Sistemi

- **UTXO Modeli**: Harcanmamış İşlem Çıktıları (UTXO) modeli eklenerek gerçekçi bir bakiye yönetim sistemi oluşturuldu
- **Bakiye Hesaplama**: Bakiyeler artık harcanmamış işlem çıktılarının toplamı olarak hesaplanıyor
- **İşlem Girdileri ve Çıktıları**: Her işlem, harcanacak UTXO'ları (girdiler) ve oluşturulacak yeni UTXO'ları (çıktılar) içeriyor
- **Para Üstü Mekanizması**: İşlemler sırasında göndericiye para üstü döndürülmesi sağlandı
- **Genesis Bloğu İyileştirmesi**: Genesis bloğu madencilik işlemi sırasında oluşturularak ilk coinlerin doğru şekilde dağıtılması sağlandı

### Modüler Yapı İyileştirmeleri

- **Block Yapısının Ayrılması**: Block yapısı ve ilgili implementasyonlar `block.rs` dosyasına taşınarak daha modüler bir yapı oluşturuldu.
- **Network Yapısının Ayrılması**: BlockchainNetwork yapısı ve ilgili implementasyonlar `network.rs` dosyasına taşındı.
- **Kod Organizasyonu**: Projenin kod organizasyonu iyileştirildi, her yapının kendi dosyasında yer alması sağlandı.
- **Kapsamlı Blok Doğrulama Sistemi**: Blockchain'lerin geçerliliği artık sadece içerik kontrolüne değil, zincir doğrulamasına ve hash karşılaştırmalarına dayanarak yapılıyor.
- **Simülasyon İyileştirmesi**: Ana simülasyon artık 5 transaction oluşturarak daha gerçekçi bir blockchain oluşturuyor, sonrasında manipülasyon denemeleri yapılıyor.

---

Bu proje, blockchain teknolojisinin temel prensiplerini anlamak ve öğrenmek için geliştirilmiş eğitim amaçlı bir simülasyondur.
