use rand::Rng;
use secp256k1::{Secp256k1, PublicKey, SecretKey};
use sha2::{Sha256, Digest};
use bs58;
use hex;

use crate::transaction::{Transaction, UTXO, TxInput, TxOutput, get_utxo_id};

#[derive(Clone, Debug)]
pub struct Wallet {
    private_key: SecretKey,
    public_key: PublicKey,
    address: String,
    balance: u64,          // Toplam bakiye
    utxos: Vec<UTXO>,      // Bu cüzdana ait harcanmamış çıktılar
}

impl Wallet {
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
            utxos: Vec::new(),
        }
    }
    
    pub fn get_address(&self) -> &str {
        &self.address
    }
    
    pub fn get_public_key(&self) -> &PublicKey {
        &self.public_key
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
        let message = secp256k1::Message::from_digest_slice(&message_hash).expect("32 bytes");
        
        // İmzala
        let signature = secp.sign_ecdsa(&message, &self.private_key);
        
        // İmzayı byte dizisine dönüştür
        signature.serialize_der().to_vec()
    }
    
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        let secp = Secp256k1::new();
        
        // İlk olarak verinin hash'ini al
        let mut hasher = Sha256::new();
        hasher.update(data);
        let message_hash = hasher.finalize();
        
        // Hash'i bir message tipine dönüştür
        let message = secp256k1::Message::from_digest_slice(&message_hash).expect("32 bytes");
        
        // İmzayı doğrula
        let signature = secp256k1::ecdsa::Signature::from_der(signature).expect("Geçerli imza");
        
        secp.verify_ecdsa(&message, &signature, &self.public_key).is_ok()
    }
    
    // Cüzdana UTXO ekle
    pub fn add_utxo(&mut self, utxo: UTXO) {
        if utxo.recipient_address == self.address {
            self.balance += utxo.amount;
            self.utxos.push(utxo);
        }
    }
    
    // Cüzdandan UTXO çıkar (harcanmış olarak işaretle)
    pub fn remove_utxo(&mut self, tx_id: &str, output_index: usize) {
        let utxo_id = get_utxo_id(tx_id, output_index);
        
        if let Some(index) = self.utxos.iter().position(|utxo| {
            get_utxo_id(&utxo.transaction_id, utxo.output_index) == utxo_id
        }) {
            let removed_utxo = self.utxos.remove(index);
            self.balance -= removed_utxo.amount;
        }
    }
    
    // Cüzdanın bakiyesini döndür
    pub fn get_balance(&self) -> u64 {
        self.balance
    }
    
    // Yeni bir işlem oluştur
    pub fn create_transaction(&self, recipient_address: &str, amount: u64) -> Option<Transaction> {
        // Bakiye kontrolü
        if amount > self.balance {
            // Yetersiz bakiye
            return None;
        }
        
        // Girdi olarak kullanılacak UTXO'ları seç
        let mut selected_utxos = Vec::new();
        let mut selected_amount = 0;
        
        for utxo in &self.utxos {
            selected_utxos.push(utxo.clone());
            selected_amount += utxo.amount;
            
            if selected_amount >= amount {
                break;
            }
        }
        
        // Girdileri oluştur
        let mut inputs = Vec::new();
        for utxo in &selected_utxos {
            let utxo_id = get_utxo_id(&utxo.transaction_id, utxo.output_index);
            
            // İmza oluştur (gerçek bir sistemde, tüm işlem verisi imzalanır)
            let signature_data = format!("{}{}{}", utxo_id, utxo.output_index, amount);
            let signature = self.sign(signature_data.as_bytes());
            
            inputs.push(TxInput {
                utxo_id,
                utxo_output_index: utxo.output_index,
                signature,
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
        let change = selected_amount - amount;
        if change > 0 {
            outputs.push(TxOutput {
                amount: change,
                recipient_address: self.address.clone(),
            });
        }
        
        // İşlemi oluştur
        Some(Transaction::new(inputs, outputs))
    }
    
    // Cüzdanın UTXO'larını güncelle (yeni bloklar geldiğinde)
    pub fn update_utxos(&mut self, transactions: &[Transaction]) {
        for tx in transactions {
            // Bu cüzdana ait harcanan UTXO'ları çıkar
            for input in &tx.inputs {
                if input.sender_address == self.address {
                    // UTXO ID'sinden transaction_id'yi çıkar
                    // Örnek: utxo_id = "abc123def0" -> tx_id = "abc123def", output_index = 0
                    let last_char = input.utxo_id.chars().last().unwrap_or('0');
                    let output_index = last_char.to_digit(10).unwrap_or(0) as usize;
                    let tx_id = &input.utxo_id[0..input.utxo_id.len()-1];
                    
                    // UTXO'nun hala cüzdanda olup olmadığını kontrol et
                    // Eğer zaten harcanmışsa (işlem oluşturulduğunda çıkarılmışsa) tekrar çıkarma
                    let utxo_exists = self.utxos.iter().any(|utxo| {
                        utxo.transaction_id == *tx_id && utxo.output_index == output_index
                    });
                    
                    if utxo_exists {
                        // UTXO hala cüzdanda, çıkar
                        self.remove_utxo(tx_id, output_index);
                    }
                }
            }
            
            // Bu cüzdana ait yeni UTXO'ları ekle
            for (i, output) in tx.outputs.iter().enumerate() {
                if output.recipient_address == self.address {
                    let utxo = UTXO {
                        transaction_id: tx.id.clone(),
                        output_index: i,
                        amount: output.amount,
                        recipient_address: self.address.clone(),
                    };
                    
                    // UTXO'nun zaten cüzdanda olup olmadığını kontrol et
                    let utxo_exists = self.utxos.iter().any(|existing_utxo| {
                        existing_utxo.transaction_id == utxo.transaction_id && 
                        existing_utxo.output_index == utxo.output_index
                    });
                    
                    if !utxo_exists {
                        // Yeni UTXO ekleniyor
                        self.add_utxo(utxo);
                    }
                }
            }
        }
    }
} 