use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutPoint {
    pub txid: String,
    pub vout: usize,
}

// UTXO (Unspent Transaction Output) yapısı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UTXO {
    pub outpoint: OutPoint,        // Bu UTXO'nun kimliği (txid + vout)
    pub amount: u64,               // Miktar (örn. satoshi cinsinden)
    pub recipient_address: String, // Alıcı adresi
}

// Transaction Input yapısı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxInput {
    pub previous_output: OutPoint, // Harcanacak UTXO'nun outpoint bilgisi
    pub signature: Vec<u8>,        // Girdi için imza
    pub public_key: Vec<u8>,       // Harcayan cüzdanın public key'i (compressed)
    pub sender_address: String,    // Gönderen adresi
}

// Transaction Output yapısı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOutput {
    pub amount: u64,               // Miktar
    pub recipient_address: String, // Alıcı adresi
}

// Transaction yapısı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: String,             // İşlem ID'si (hash)
    pub inputs: Vec<TxInput>,   // Girdiler
    pub outputs: Vec<TxOutput>, // Çıktılar
    pub timestamp: u64,         // Zaman damgası
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
            inputs: Vec::new(), // Coinbase işleminde girdi yok
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
            hasher.update(input.previous_output.txid.as_bytes());
            hasher.update(input.previous_output.vout.to_le_bytes());
            hasher.update(&input.public_key);
            hasher.update(input.sender_address.as_bytes());
        }

        // Çıktıları hash'e ekle
        for output in &self.outputs {
            let output_data = format!("{}{}", output.amount, output.recipient_address);
            hasher.update(output_data.as_bytes());
        }

        // Zaman damgasını ekle
        hasher.update(self.timestamp.to_string().as_bytes());

        // Hash'i hesapla ve döndür
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.is_empty() && !self.outputs.is_empty()
    }

    // Her input için imzalanacak veriyi üret
    pub fn signing_payload(&self, input_index: usize) -> Option<Vec<u8>> {
        if input_index >= self.inputs.len() {
            return None;
        }

        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update((input_index as u64).to_le_bytes());

        for input in &self.inputs {
            hasher.update(input.previous_output.txid.as_bytes());
            hasher.update(input.previous_output.vout.to_le_bytes());
            hasher.update(&input.public_key);
        }

        for output in &self.outputs {
            hasher.update(output.amount.to_le_bytes());
            hasher.update(output.recipient_address.as_bytes());
        }

        Some(hasher.finalize().to_vec())
    }

    // İşlemin toplam girdi miktarını hesapla
    pub fn get_total_input_amount(&self, utxo_set: &HashMap<OutPoint, UTXO>) -> u64 {
        let mut total = 0;

        for input in &self.inputs {
            if let Some(utxo) = utxo_set.get(&input.previous_output) {
                total += utxo.amount;
            }
        }

        total
    }

    // İşlemin toplam çıktı miktarını hesapla
    pub fn get_total_output_amount(&self) -> u64 {
        self.outputs.iter().map(|output| output.amount).sum()
    }

    // İşlem ücretini hesapla (input - output)
    pub fn calculate_fee(&self, utxo_set: &HashMap<OutPoint, UTXO>) -> Option<u64> {
        if self.is_coinbase() {
            return None;
        }

        let total_input = self.get_total_input_amount(utxo_set);
        let total_output = self.get_total_output_amount();
        total_input.checked_sub(total_output)
    }

    // İşlemin geçerli olup olmadığını kontrol et
    pub fn is_valid(&self, utxo_set: &HashMap<OutPoint, UTXO>) -> bool {
        if self.outputs.is_empty() {
            return false;
        }

        if self.outputs.iter().any(|output| output.amount == 0) {
            return false;
        }

        // Coinbase işlemi her zaman geçerlidir
        if self.is_coinbase() {
            return true;
        }

        if self.id != self.calculate_hash() {
            return false;
        }

        let mut seen_outpoints = HashSet::new();
        for input in &self.inputs {
            let outpoint = input.previous_output.clone();
            if !seen_outpoints.insert(outpoint) {
                return false;
            }
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
            write!(
                f,
                "  [{}] Outpoint: {}:{}, Gönderen: {}\n",
                i, input.previous_output.txid, input.previous_output.vout, input.sender_address
            )?;
        }

        write!(f, "Outputs:\n")?;
        for (i, output) in self.outputs.iter().enumerate() {
            write!(
                f,
                "  [{}] Miktar: {} coin, Alıcı: {}\n",
                i,
                output.amount as f64 / 100_000_000.0,
                output.recipient_address
            )?;
        }

        Ok(())
    }
}
