use std::fmt;
use sha2::{Sha256, Digest};

// Block yapısı
#[derive(Debug, Clone)]
pub struct Block {
    pub index: usize,
    pub timestamp: u64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: usize, timestamp: u64, data: String, previous_hash: String) -> Self {
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    // Block'un hash'ini hesapla
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let contents = format!("{}{}{}{}{}", self.index, self.timestamp, self.data, self.previous_hash, self.nonce);
        hasher.update(contents.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    // Proof of Work (basit bir zorluk seviyesi)
    pub fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty); // Hedef: belirli sayıda 0 ile başlayan hash
        
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        
        println!("Block mined! Nonce: {}, Hash: {}", self.nonce, self.hash);
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Block #{}: [Previous Hash: {}, Hash: {}, Data: {}]", 
               self.index, self.previous_hash, self.hash, self.data)
    }
} 