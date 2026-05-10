use bs58;
use rand::Rng;
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::transaction::{OutPoint, Transaction, TxInput, TxOutput, UTXO};

#[derive(Clone, Debug)]
pub struct Wallet {
    private_key: SecretKey,
    public_key: PublicKey,
    address: String,
    balance: u64,                   // Toplam bakiye
    utxos: HashMap<OutPoint, UTXO>, // Bu cüzdana ait harcanmamış çıktılar
}

impl Wallet {
    const DEFAULT_TRANSACTION_FEE: u64 = 1_000;

    pub fn new() -> Self {
        // 1. Özel anahtar oluştur (256 bit rastgele sayı)
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        // 32 byte'lık rastgele bir sayı oluşturup SecretKey'e dönüştür
        let random_bytes: [u8; 32] = core::array::from_fn(|_| rng.gen());
        let secret_key = SecretKey::from_slice(&random_bytes).expect("32 bytes secret key");

        // 2. Genel anahtarı elde et (ECDSA kullanılır)
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        // 3-6. Adres oluştur
        let address = Self::generate_address(&public_key);

        Wallet {
            private_key: secret_key,
            public_key,
            address,
            balance: 0,
            utxos: HashMap::new(),
        }
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn get_public_key_bytes(&self) -> Vec<u8> {
        self.public_key.serialize().to_vec()
    }

    fn generate_address(public_key: &PublicKey) -> String {
        // Public key'i hash'le (Bitcoin'den farklı olarak sadece tek bir SHA-256 kullan)
        let public_key_bytes = public_key.serialize();

        // SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&public_key_bytes);
        let hash_result = hasher.finalize();

        // Version byte ekle (0x00)
        let mut versioned_hash = vec![0x00];
        versioned_hash.extend_from_slice(&hash_result[..20]); // İlk 20 byte'ı al (RIPEMD160 gibi 20 byte'a sıkıştırılmış)

        // Checksum ekle (verinin SHA-256'sının SHA-256'sından ilk 4 byte)
        let mut checksum_hasher1 = Sha256::new();
        checksum_hasher1.update(&versioned_hash);
        let checksum_result1 = checksum_hasher1.finalize();

        let mut checksum_hasher2 = Sha256::new();
        checksum_hasher2.update(&checksum_result1);
        let checksum_result2 = checksum_hasher2.finalize();

        // İlk 4 byte'ı al
        let checksum = &checksum_result2[0..4];

        // Versiyonlu hash ile checksum'ı birleştir
        let mut address_bytes = versioned_hash.clone();
        address_bytes.extend_from_slice(checksum);

        // Base58Check kodlaması yap
        bs58::encode(address_bytes).into_string()
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let secp = Secp256k1::new();

        // İlk olarak verinin hash'ini al
        let mut hasher = Sha256::new();
        hasher.update(data);
        let message_hash = hasher.finalize();

        // Hash'i bir message tipine dönüştür
        let message = Message::from_digest_slice(&message_hash).expect("32 bytes");

        // İmzala
        let signature = secp.sign_ecdsa(&message, &self.private_key);

        // İmzayı byte dizisine dönüştür
        signature.serialize_der().to_vec()
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        Self::verify_with_public_key(data, signature, &self.public_key)
    }

    pub fn verify_with_public_key(data: &[u8], signature: &[u8], public_key: &PublicKey) -> bool {
        let secp = Secp256k1::new();

        // İlk olarak verinin hash'ini al
        let mut hasher = Sha256::new();
        hasher.update(data);
        let message_hash = hasher.finalize();

        // Hash'i bir message tipine dönüştür
        let message = Message::from_digest_slice(&message_hash).expect("32 bytes");

        // İmzayı doğrula
        let Ok(signature) = Signature::from_der(signature) else {
            return false;
        };

        secp.verify_ecdsa(&message, &signature, public_key).is_ok()
    }

    pub fn verify_with_public_key_bytes(
        data: &[u8],
        signature: &[u8],
        public_key_bytes: &[u8],
    ) -> bool {
        let Ok(public_key) = PublicKey::from_slice(public_key_bytes) else {
            return false;
        };
        Self::verify_with_public_key(data, signature, &public_key)
    }

    pub fn public_key_bytes_to_address(public_key_bytes: &[u8]) -> Option<String> {
        let Ok(public_key) = PublicKey::from_slice(public_key_bytes) else {
            return None;
        };
        Some(Self::generate_address(&public_key))
    }

    // Cüzdana UTXO ekle
    pub fn add_utxo(&mut self, utxo: UTXO) {
        if utxo.recipient_address == self.address && !self.utxos.contains_key(&utxo.outpoint) {
            self.balance += utxo.amount;
            self.utxos.insert(utxo.outpoint.clone(), utxo);
        }
    }

    // Cüzdandan UTXO çıkar (harcanmış olarak işaretle)
    pub fn remove_utxo(&mut self, outpoint: &OutPoint) {
        if let Some(removed_utxo) = self.utxos.remove(outpoint) {
            self.balance -= removed_utxo.amount;
        }
    }

    // Cüzdanın bakiyesini döndür
    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    // Cüzdanın durumunu (UTXO + bakiye) global UTXO setten yeniden kur
    // Private/public key ve adres korunur.
    pub fn rebuild_from_utxo_set(&mut self, global_utxo_set: &HashMap<OutPoint, UTXO>) {
        self.utxos.clear();
        self.balance = 0;

        for (outpoint, utxo) in global_utxo_set {
            if utxo.recipient_address == self.address {
                self.utxos.insert(outpoint.clone(), utxo.clone());
                self.balance = self.balance.saturating_add(utxo.amount);
            }
        }
    }

    // Yeni bir işlem oluştur
    pub fn create_transaction(&self, recipient_address: &str, amount: u64) -> Option<Transaction> {
        self.create_transaction_with_fee(recipient_address, amount, Self::DEFAULT_TRANSACTION_FEE)
    }

    // Belirli bir fee ile işlem oluştur
    pub fn create_transaction_with_fee(
        &self,
        recipient_address: &str,
        amount: u64,
        fee: u64,
    ) -> Option<Transaction> {
        let required_total = amount.checked_add(fee)?;

        // Bakiye kontrolü
        if required_total > self.balance {
            // Yetersiz bakiye
            return None;
        }

        // Girdi olarak kullanılacak UTXO'ları seç
        let mut selected_utxos = Vec::new();
        let mut selected_amount = 0;

        for utxo in self.utxos.values() {
            selected_utxos.push(utxo.clone());
            selected_amount += utxo.amount;

            if selected_amount >= required_total {
                break;
            }
        }

        // Girdileri oluştur
        let mut inputs = Vec::new();
        let public_key_bytes = self.get_public_key_bytes();
        for utxo in &selected_utxos {
            inputs.push(TxInput {
                previous_output: utxo.outpoint.clone(),
                signature: Vec::new(),
                public_key: public_key_bytes.clone(),
                sender_address: self.address.clone(),
            });
        }

        // Çıktıları oluştur
        let mut outputs = Vec::new();

        // Alıcıya gönderilecek miktar
        outputs.push(TxOutput {
            amount,
            recipient_address: recipient_address.to_string(),
        });

        // Para üstü (eğer varsa)
        let change = selected_amount - required_total;
        if change > 0 {
            outputs.push(TxOutput {
                amount: change,
                recipient_address: self.address.clone(),
            });
        }

        // İşlemi oluştur
        let mut transaction = Transaction::new(inputs, outputs);
        for i in 0..transaction.inputs.len() {
            let signature_payload = transaction.signing_payload(i)?;
            transaction.inputs[i].signature = self.sign(&signature_payload);
        }

        Some(transaction)
    }

    // Cüzdanın UTXO'larını güncelle (yeni bloklar geldiğinde)
    pub fn update_utxos(&mut self, transactions: &[Transaction]) {
        for tx in transactions {
            // Bu cüzdana ait harcanan UTXO'ları çıkar
            for input in &tx.inputs {
                if input.sender_address == self.address {
                    // UTXO'nun hala cüzdanda olup olmadığını kontrol et
                    // Eğer zaten harcanmışsa (işlem oluşturulduğunda çıkarılmışsa) tekrar çıkarma
                    if self.utxos.contains_key(&input.previous_output) {
                        // UTXO hala cüzdanda, çıkar
                        self.remove_utxo(&input.previous_output);
                    }
                }
            }

            // Bu cüzdana ait yeni UTXO'ları ekle
            for (i, output) in tx.outputs.iter().enumerate() {
                if output.recipient_address == self.address {
                    let utxo = UTXO {
                        outpoint: OutPoint {
                            txid: tx.id.clone(),
                            vout: i,
                        },
                        amount: output.amount,
                        recipient_address: self.address.clone(),
                    };

                    // UTXO'nun zaten cüzdanda olup olmadığını kontrol et
                    if !self.utxos.contains_key(&utxo.outpoint) {
                        // Yeni UTXO ekleniyor
                        self.add_utxo(utxo);
                    }
                }
            }
        }
    }
}
