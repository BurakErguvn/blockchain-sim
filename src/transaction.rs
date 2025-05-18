use sha2::{Sha256, Digest};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use hex;

// UTXO (Unspent Transaction Output) yapısı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UTXO {
    pub transaction_id: String,  // Bu UTXO'nun ait olduğu işlemin ID'si
    pub output_index: usize,     // İşlemdeki çıktı indeksi
    pub amount: u64,             // Miktar (örn. satoshi cinsinden)
    pub recipient_address: String, // Alıcı adresi
}

// Transaction Input yapısı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxInput {
    pub utxo_id: String,         // Harcanacak UTXO'nun ID'si (transaction_id + output_index)
    pub utxo_output_index: usize, // UTXO'nun çıktı indeksi
    pub signature: Vec<u8>,      // Girdi için imza
    pub sender_address: String,  // Gönderen adresi
}

// Transaction Output yapısı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOutput {
    pub amount: u64,             // Miktar
    pub recipient_address: String, // Alıcı adresi
}

// Transaction yapısı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: String,              // İşlem ID'si (hash)
    pub inputs: Vec<TxInput>,    // Girdiler
    pub outputs: Vec<TxOutput>,  // Çıktılar
    pub timestamp: u64,          // Zaman damgası
}

impl Transaction {
    // Yeni bir transaction oluştur
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut tx = Transaction {
            id: String::new(),
            inputs,
            outputs,
            timestamp,
        };
        
        // Transaction ID'sini hesapla
        tx.id = tx.calculate_hash();
        tx
    }
    
    // Coinbase transaction (madencilik ödülü) oluştur
    pub fn new_coinbase(recipient_address: String, amount: u64) -> Self {
        // Her coinbase işlemi için benzersiz bir timestamp kullan
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Rasgele bir nonce ekleyerek her coinbase işleminin benzersiz olmasını sağla
        let nonce = rand::random::<u64>();
        
        // Coinbase işleminde girdi yoktur, sadece çıktı vardır
        let outputs = vec![TxOutput {
            amount,
            recipient_address,
        }];
        
        let mut tx = Transaction {
            id: String::new(),
            inputs: Vec::new(),  // Coinbase işleminde girdi yok
            outputs,
            timestamp,
        };
        
        // ID hesaplanırken nonce'u da dahil et
        tx.id = format!("{}{}", tx.calculate_hash(), nonce);
        tx
    }
    
    // Transaction hash'ini hesapla
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        
        // Girdileri hash'e ekle
        for input in &self.inputs {
            let input_data = format!("{}{}{}", 
                input.utxo_id, 
                input.utxo_output_index,
                input.sender_address
            );
            hasher.update(input_data.as_bytes());
        }
        
        // Çıktıları hash'e ekle
        for output in &self.outputs {
            let output_data = format!("{}{}", 
                output.amount, 
                output.recipient_address
            );
            hasher.update(output_data.as_bytes());
        }
        
        // Zaman damgasını ekle
        hasher.update(self.timestamp.to_string().as_bytes());
        
        // Hash'i hesapla ve döndür
        let result = hasher.finalize();
        format!("{:x}", result)
    }
    
    // İşlemin toplam girdi miktarını hesapla
    pub fn get_total_input_amount(&self, utxo_set: &[UTXO]) -> u64 {
        let mut total = 0;
        
        for input in &self.inputs {
            // UTXO setinde bu girdiyle eşleşen UTXO'yu bul
            for utxo in utxo_set {
                let utxo_id = format!("{}{}", utxo.transaction_id, utxo.output_index);
                if utxo_id == input.utxo_id && utxo.output_index == input.utxo_output_index {
                    total += utxo.amount;
                    break;
                }
            }
        }
        
        total
    }
    
    // İşlemin toplam çıktı miktarını hesapla
    pub fn get_total_output_amount(&self) -> u64 {
        self.outputs.iter().map(|output| output.amount).sum()
    }
    
    // İşlemin geçerli olup olmadığını kontrol et
    pub fn is_valid(&self, utxo_set: &[UTXO]) -> bool {
        // Coinbase işlemi her zaman geçerlidir
        if self.inputs.is_empty() && !self.outputs.is_empty() {
            return true;
        }
        
        // Toplam girdi ve çıktı miktarlarını kontrol et
        let total_input = self.get_total_input_amount(utxo_set);
        let total_output = self.get_total_output_amount();
        
        // Çıktı miktarı girdi miktarından büyük olamaz
        if total_output > total_input {
            // Geçersiz işlem: Çıktı miktarı girdi miktarından büyük
            return false;
        }
        
        true
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction ID: {}\n", self.id)?;
        
        write!(f, "Inputs:\n")?;
        for (i, input) in self.inputs.iter().enumerate() {
            write!(f, "  [{}] UTXO: {}, Gönderen: {}\n", 
                i, input.utxo_id, input.sender_address)?;
        }
        
        write!(f, "Outputs:\n")?;
        for (i, output) in self.outputs.iter().enumerate() {
            write!(f, "  [{}] Miktar: {} coin, Alıcı: {}\n", 
                i, output.amount as f64 / 100_000_000.0, output.recipient_address)?;
        }
        
        Ok(())
    }
}

// UTXO'ları yönetmek için yardımcı fonksiyonlar
pub fn get_utxo_id(tx_id: &str, output_index: usize) -> String {
    // UTXO ID'si, transaction ID ve output index'in birleşiminden oluşur
    // Örnek: tx_id = "abc123def", output_index = 0 -> utxo_id = "abc123def0"
    format!("{}{}", tx_id, output_index)
}
