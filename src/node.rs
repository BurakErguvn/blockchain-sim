use rand::Rng;
use rand::thread_rng;
use std::collections::HashMap;
use std::fmt;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

// Block modülünü kullan
use crate::block::Block;
use crate::wallet::Wallet;

//Node sınıfı
#[derive(Debug)]
pub struct Node {
    pub id:usize,
    pub hash:String,
    pub connections: Vec<usize>, //Bağlı nodeların id'leri
    pub is_validator: bool,
    pub valinfo: String,
    pub blockchain: Vec<Block>, // Blok zinciri
    pub wallet: Wallet, // Cüzdan
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node ID: {}, Address: {}, Hash: {}, Validator: {}, Connections: {:?}, Blockchain Length: {}", 
            self.id, self.wallet.get_address(), self.hash, self.is_validator, self.connections, self.blockchain.len())
    }
}

impl Node {
    pub fn new (id:usize) -> Self {
        // Genesis bloğunu oluştur
        let genesis_block = Block::new(
            0, 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            "Genesis Block".to_string(),
            "0".to_string(),
        );
        
        let mut blockchain = Vec::new();
        blockchain.push(genesis_block);
        
        Node {
            id,
            connections: Vec::new(),
            is_validator: false,
            valinfo: String::new(),
            hash: String::new(),
            blockchain,
            wallet: Wallet::new(), // Yeni bir cüzdan oluştur
        }
    }

    // Cüzdan adresini almak için yeni fonksiyon
    pub fn get_address(&self) -> &str {
        self.wallet.get_address()
    }

    //Gelen valinfoyu hash'e çeviren fonksiyon
    pub fn process_valinfo(&mut self, valinfo: &str) -> String {
        if self.is_validator {
            //SHA-256 hash fonksiyonu
            let mut hasher = Sha256::new();
            hasher.update(valinfo.as_bytes());
            let result = hasher.finalize();
            let hash = format!("{:x}", result);
            
            self.valinfo = valinfo.to_string();
            self.hash = hash.clone();
            hash
        }else {
            println!("Node {} is not a validator.", self.id);
            String::new()
        }
    }

    // Transaction'ı imzala
    pub fn sign_transaction(&self, transaction_data: &str) -> Vec<u8> {
        self.wallet.sign(transaction_data.as_bytes())
    }
    
    // İmzayı doğrula
    pub fn verify_transaction(&self, transaction_data: &str, signature: &[u8]) -> bool {
        self.wallet.verify(transaction_data.as_bytes(), signature)
    }

    //Hash değerini güncelleme
    pub fn update_hash(&mut self, new_hash: String) {
        self.hash = new_hash;
    }

    //Node bağlantısı ekleme
    pub fn add_connection(&mut self, node_id: usize) {
        if node_id == self.id {
            return;
        }
        
        if self.connections.contains(&node_id) {
            println!("Info: Node {} is already connected to Node {}.", self.id, node_id);
            return;
        }
        
        self.connections.push(node_id);
    }
    
    // Yeni bir blok ekle
    pub fn add_block(&mut self, data: String, difficulty: usize) -> Block {
        let last_block = self.blockchain.last().unwrap();
        let new_index = last_block.index + 1;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        let mut new_block = Block::new(
            new_index,
            timestamp,
            data,
            last_block.hash.clone(),
        );
        
        // Blok madenciliği
        new_block.mine_block(difficulty);
        
        self.blockchain.push(new_block.clone());
        new_block
    }
    
    // Blockchain'i güncelle
    pub fn update_blockchain(&mut self, blockchain: Vec<Block>) {
        if blockchain.len() > self.blockchain.len() {
            // Yeni zincir daha uzunsa, mevcut zinciri değiştir
            self.blockchain = blockchain;
        } else if blockchain.len() == self.blockchain.len() {
            // Aynı uzunluktaki zincirler arasında farklılık ve geçerlilik kontrolü yap
            
            // Önce her iki blockchain'in de kendi içinde geçerli olup olmadığını kontrol et
            let self_valid = self.is_chain_valid();
            
            // Gelen blockchain'i geçici bir Node'a ekleyip geçerli mi diye kontrol et
            let mut temp_node = Node::new(999); // Geçici bir ID ile node oluştur
            temp_node.blockchain = blockchain.clone();
            let incoming_valid = temp_node.is_chain_valid();
            
            // Eğer mevcut zincir geçersiz ve gelen zincir geçerliyse, değiştir
            if !self_valid && incoming_valid {
                println!("Current chain is invalid, replacing with valid incoming chain.");
                self.blockchain = blockchain;
                return;
            }
            
            // Eğer her iki zincir de geçerliyse, hash'leri karşılaştır
            if self_valid && incoming_valid {
                // Son blok hash'leri farklı mı?
                if let (Some(self_last), Some(incoming_last)) = (self.blockchain.last(), blockchain.last()) {
                    if self_last.hash != incoming_last.hash {
                        // Hash'ler farklı ama her iki zincir de kendi içinde geçerli
                        // Bu durumda çoğunluk kuralını takip etmeliyiz (bunu BlockchainNetwork sınıfı yapacak)
                        println!("Both chains are valid but have different last blocks.");
                        return;
                    }
                }
            }
            
            // Eğer mevcut zincir geçersiz ve gelen zincir de geçersizse, hiçbir şey yapma
            // Bu durumda blockchain ağı başka bir düğümden geçerli bir zincir alabilir
            if !self_valid && !incoming_valid {
                println!("Both current and incoming chains are invalid, no update performed.");
                return;
            }
        }
    }

    // Blockchain'in geçerliliğini kontrol et
    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.blockchain.len() {
            let current_block = &self.blockchain[i];
            let previous_block = &self.blockchain[i - 1];
            
            // Hash doğrulaması
            if current_block.hash != current_block.calculate_hash() {
                return false;
            }
            
            // Previous hash doğrulaması
            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }
}