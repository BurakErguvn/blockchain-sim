use std::fmt;
use sha2::{Sha256, Digest};

// Transaction modülünü kullan
use crate::transaction::Transaction;

// Block yapısı
#[derive(Debug, Clone)]
pub struct Block {
    pub index: usize,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,  // İşlemler listesi
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub merkle_root: String,            // Merkle kök hash'i
}

impl Block {
    pub fn new(index: usize, timestamp: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            merkle_root: String::new(),
        };
        
        // Merkle kök hash'ini hesapla
        block.merkle_root = block.calculate_merkle_root();
        
        // Blok hash'ini hesapla
        block.hash = block.calculate_hash();
        block
    }

    // Block'un hash'ini hesapla
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let contents = format!("{}{}{}{}{}", 
            self.index, 
            self.timestamp, 
            self.merkle_root, 
            self.previous_hash, 
            self.nonce
        );
        hasher.update(contents.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
    
    // Merkle kök hash'ini hesapla
    pub fn calculate_merkle_root(&self) -> String {
        if self.transactions.is_empty() {
            return "0".to_string();
        }
        
        // Önce tüm işlemlerin hash'lerini al
        let mut hashes: Vec<String> = self.transactions
            .iter()
            .map(|tx| tx.id.clone())
            .collect();
        
        // Tek sayıda hash varsa, son hash'i tekrarla
        if hashes.len() % 2 == 1 {
            hashes.push(hashes.last().unwrap().clone());
        }
        
        // Merkle ağacını oluştur
        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();
            
            // İkişerli grupla ve hash'le
            for i in (0..hashes.len()).step_by(2) {
                let mut hasher = Sha256::new();
                let combined = format!("{}{}", hashes[i], hashes[i + 1]);
                hasher.update(combined.as_bytes());
                let result = hasher.finalize();
                new_hashes.push(format!("{:x}", result));
            }
            
            hashes = new_hashes;
        }
        
        hashes[0].clone()
    }

    // Proof of Work (basit bir zorluk seviyesi)
    pub fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty); // Hedef: belirli sayıda 0 ile başlayan hash
        
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        
        // Block mined
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Block #{}: [Previous Hash: {}, Hash: {}, Transactions: {}]", 
               self.index, self.previous_hash, self.hash, self.transactions.len())
    }
} 