use rand::Rng;
use rand::thread_rng;
use std::collections::HashMap;
use std::fmt;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

// Block ve Node modüllerini kullan
use crate::block::Block;
use crate::node::Node;

pub struct BlockchainNetwork {
    nodes: HashMap<usize, Node>,
    current_validator_id: Option<usize>,
    difficulty: usize, // Madencilik zorluğu
}

impl BlockchainNetwork {
    pub fn new() -> Self {
        BlockchainNetwork {
            nodes: HashMap::new(),
            current_validator_id: None,
            difficulty: 2, // Varsayılan zorluk seviyesi
        }
    }

    //Yeni bir node ekleme
    pub fn add_node(&mut self) -> usize {
        let id = self.nodes.len();
        let node = Node::new(id);
        self.nodes.insert(id, node);
        id
    }

    //İki node arasında bağlantı oluşturma
    pub fn connect_nodes(&mut self, node1_id: usize, node2_id: usize) {
        // Aynı node ise bağlantı kurma
        if node1_id == node2_id {
            return;
        }

        if let Some(node1) = self.nodes.get_mut(&node1_id) {
            node1.add_connection(node2_id);
        } else {
            println!("Warning: Node {} not found.", node1_id);
        }
        
        if let Some(node2) = self.nodes.get_mut(&node2_id) {
            node2.add_connection(node1_id);
        } else {
            println!("Warning: Node {} not found.", node2_id);
        }
    }

    // Rasgele bir validator seç
    pub fn select_random_validator(&mut self) {
        if self.nodes.is_empty() {
            println!("No nodes in the network.");
            return;
        }

        //önce tüm node'ları validator olmaktan çıkar
        for node in self.nodes.values_mut() {
            node.is_validator = false;
        }  

        //Rasgele bir node seç ve validator yap
        let mut rng = rand::thread_rng();
        let validator_id = rng.gen_range(0..self.nodes.len());

        if let Some(node) = self.nodes.get_mut(&validator_id) {
            node.is_validator = true;
            self.current_validator_id = Some(validator_id);
            println!("Node {} is selected as the new validator.", validator_id);
        }
    }

    //Yeni bir işlem (valinfo) oluştur ve doğrula
    pub fn create_transaction(&mut self, valinfo: &str) {
        if let Some(validator_id) = self.current_validator_id {
            // Önce hash'i oluştur ve blockchain ekle
            let blockchain_clone;
            
            {
                let validator = match self.nodes.get_mut(&validator_id) {
                    Some(v) => v,
                    None => {
                        println!("Validator not found.");
                        return;
                    }
                };
                
                // Validator valinfo'yu işler
                let transaction_hash = validator.process_valinfo(valinfo);
                println!("Validator {} processed transaction with hash: {}", validator_id, transaction_hash);
                
                // Yeni bir blok oluştur
                let new_block = validator.add_block(valinfo.to_string(), self.difficulty);
                println!("Validator {} created a new block: {}", validator_id, new_block);
                
                // Madencilik sonucu oluşan hash değerini alıyoruz
                let mined_hash = new_block.hash.clone();
                
                // Önce blockchain'i clone'la
                blockchain_clone = validator.blockchain.clone();
                
                // Sonra mined hash'i validator'a ata
                validator.hash = mined_hash.clone();
                
                // Sonra hash'i broadcast et (madencilik sonucu oluşan hash)
                self.broadcast_hash(mined_hash);
            }
            
            // Blockchain'i broadcast et
            self.broadcast_blockchain(blockchain_clone);
            
            // Validator'ın yetkisini kaldır
            if let Some(node) = self.nodes.get_mut(&validator_id) {
                node.is_validator = false;
                println!("Node {}'s validator status has been revoked after creating a block.", validator_id);
            }
            self.current_validator_id = None;
            
            println!("A new validator will need to be selected for the next transaction.");
        } else {
            println!("No validator selected.");
        }
    }

    //Hash'i tüm bağlı node'lara gönder
    pub fn broadcast_hash(&mut self, hash: String){
        for node in self.nodes.values_mut()  {
            node.update_hash(hash.clone());
        }
        println!("Broadcasted hash {} to all nodes.", hash);
    }
    
    // Blockchain'i tüm node'lara yayınla
    pub fn broadcast_blockchain(&mut self, blockchain: Vec<Block>) {
        for node in self.nodes.values_mut() {
            node.update_blockchain(blockchain.clone());
        }
        println!("Broadcasted blockchain with {} blocks to all nodes.", blockchain.len());
    }

    //Bir node'un hash'ini manipüle etmeyi dene
    pub fn try_manipulate_hash(&mut self, node_id : usize, fake_hash: String) -> bool {
        if let Some(validator_id) = self.current_validator_id {
            if node_id == validator_id {
                //Eğer validator hash'i değiştirirse, bu yeni hash olur
                if let Some(node) = self.nodes.get_mut(&node_id){
                    node.hash = fake_hash.clone();
                    self.broadcast_hash(fake_hash.clone());
                    println!("Validator has changed the hash. New hash: {}", fake_hash);
                    return true;
                }
            }
        } else {
            //Validator olmayan bir node hash'i değiştirmeye çalışırsa
            if let Some(_node) = self.nodes.get_mut(&node_id) {
                let orginal_hash = fake_hash.clone();
                println!("Node {} tried to manipulate the hash: {} -> {}", node_id, orginal_hash, fake_hash);

                //Oylama yap - %51 konsensüs gerekli
                let total_nodes = self.nodes.len();
                let mut matching_hash_count = 0;
                for (id,other_node) in &self.nodes {
                    if id != &node_id && other_node.hash == orginal_hash {
                        matching_hash_count += 1;
                    }
                    
                }

                //Konsensüs kontrolü
                if matching_hash_count > total_nodes / 2 {
                    //Konsensüs sağlandı, hash düzeltilecek
                    if let Some(node) = self.nodes.get_mut(&node_id) {
                        node.hash = orginal_hash.clone();
                        println!("Consensus achieved! Fixed hash {} of node {}", orginal_hash, node_id);
                        return false;
                    }
                } else {
                    //Manipülasyon başarılı
                    println!("WARNING: Manipulation successful! Node {} has hash {}", node_id, fake_hash);
                    return true;
                }
            }
        }
        false
    }
    
    // Bir node'un blockchain'ini manipüle etmeyi dene
    pub fn try_manipulate_blockchain(&mut self, node_id: usize, custom_hash: Option<String>) -> bool {
        // Manipüle edilecek node'u al ve blockchain'i değiştir
        let mut manipulation_successful = false;
        let mut manipulated_chain_is_valid = true;
        let mut last_block_index = 0;
        let difficulty = self.difficulty; // Zorluk seviyesini al
        
        {
            let node = match self.nodes.get_mut(&node_id) {
                Some(n) => n,
                None => return false,
            };
            
            // En son bloğu değiştirmeye çalış
            if let Some(last_block) = node.blockchain.last_mut() {
                last_block_index = last_block.index;
                let original_data = last_block.data.clone();
                
                // Veriyi değiştir
                last_block.data = format!("Manipulated: {}", original_data);
                
                // PoW kurallarına uygun olarak yeni hash ve nonce hesapla
                println!("Node {} is trying to manipulate blockchain with PoW. Mining new block...", node_id);
                
                // Custom hash verilmişse, direkt onu kullan
                if let Some(hash) = custom_hash {
                    println!("Using custom hash for manipulation: {}", hash);
                    last_block.hash = hash;
                    
                    // Hash zorluğa uygun mu kontrol et
                    let target = "0".repeat(difficulty);
                    if !last_block.hash.starts_with(&target) {
                        println!("WARNING: Custom hash does not meet difficulty requirement (should start with '{}').", target);
                        println!("Applying PoW to generate a valid hash...");
                        // Nonce'u sıfırla ve zorluğa uygun yeni hash hesapla
                        last_block.nonce = 0;
                        let start_time = std::time::Instant::now();
                        while !last_block.hash.starts_with(&target) {
                            last_block.nonce += 1;
                            if last_block.nonce % 100 == 0 {
                                println!("  Mining attempt: nonce={}, current hash={}", last_block.nonce, &last_block.hash[..10]);
                            }
                            last_block.hash = last_block.calculate_hash();
                        }
                        let elapsed = start_time.elapsed();
                        println!("Generated valid hash: {} (took {:?})", last_block.hash, elapsed);
                    } else {
                        println!("Custom hash meets difficulty requirement.");
                    }
                } else {
                    // Nonce'u sıfırla ve zorluğa uygun yeni hash hesapla
                    last_block.nonce = 0;
                    let target = "0".repeat(difficulty);
                    
                    // PoW algoritması ile yeni hash hesapla
                    let start_time = std::time::Instant::now();
                    while !last_block.hash.starts_with(&target) {
                        last_block.nonce += 1;
                        if last_block.nonce % 100 == 0 {
                            println!("  Mining attempt: nonce={}, current hash={}", last_block.nonce, &last_block.hash[..10]);
                        }
                        last_block.hash = last_block.calculate_hash();
                    }
                    let elapsed = start_time.elapsed();
                    println!("Node {} manipulated block #{} successfully with nonce {} and hash: {} (took {:?})", 
                             node_id, last_block.index, last_block.nonce, last_block.hash, elapsed);
                }
                
                // Blockchain'in geçerliliğini kontrol et
                manipulated_chain_is_valid = node.is_chain_valid();
                if !manipulated_chain_is_valid {
                    println!("WARNING: Despite mining effort, Node {}'s blockchain is still invalid!", node_id);
                } else {
                    println!("Node {}'s manipulated blockchain is valid according to PoW rules.", node_id);
                }
            }
        }
        
        // Diğer node'ların geçerlilik durumunu kontrol et ve geçerli blockchain'leri topla
        let mut valid_chains_count = 0;
        let total_nodes = self.nodes.len();
        let mut valid_blockchain_source = None;
        
        for (id, node) in &self.nodes {
            if *id != node_id && node.is_chain_valid() {
                valid_chains_count += 1;
                
                if valid_blockchain_source.is_none() {
                    valid_blockchain_source = Some((*id, node.blockchain.clone()));
                }
            }
        }
        
        // Eğer geçerli zincirler çoğunluktaysa manipülasyon başarısız olur (konsensüs mekanizması)
        if valid_chains_count > (total_nodes / 2) {
            println!("Manipulation detected! Despite valid PoW, Node {}'s blockchain will be rejected by consensus.", node_id);
            
            // Geçerli blockchain'i al
            let valid_blockchain = if let Some((source_id, blockchain)) = valid_blockchain_source {
                // Geçerli bir zinciri manipüle edilen node'a gönder
                if let Some(node) = self.nodes.get_mut(&node_id) {
                    node.update_blockchain(blockchain.clone());
                    println!("Node {}'s blockchain restored from Node {}.", node_id, source_id);
                }
                Some(blockchain)
            } else {
                None
            };
            
            // Tüm ağa geçerli blockchain'i broadcast et
            if let Some(blockchain) = valid_blockchain {
                self.broadcast_blockchain(blockchain);
                println!("Valid blockchain broadcasted to all nodes to ensure consistency.");
            }
            
            manipulation_successful = false;
        } else {
            println!("WARNING: Manipulation successful! Node {}'s manipulated blockchain (with valid PoW) is accepted.", node_id);
            manipulation_successful = true;
        }
        
        manipulation_successful
    }

    //Ağın durumunu görüntüle
    pub fn print_network_state(&self){
        println!("\n--- BLOCKCHAIN NETWORK STATE ---");
        for node in self.nodes.values() {
            println!("{}", node);
        }
        println!("---------------------------------\n");
    }
    
    // Belirli bir node'un blockchain'ini görüntüle
    pub fn print_blockchain(&self, node_id: usize) {
        if let Some(node) = self.nodes.get(&node_id) {
            println!("\n--- BLOCKCHAIN FROM NODE {} ---", node_id);
            for block in &node.blockchain {
                println!("{}", block);
            }
            println!("---------------------------------\n");
        }
    }

    //Ağdaki node sayısını döndür
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    //Şu anki validator id'sini döndür
    pub fn current_val_id(&self) -> Option<usize> {
        self.current_validator_id
    }
    
    // Zorluk seviyesini ayarla
    pub fn set_difficulty(&mut self, difficulty: usize) {
        self.difficulty = difficulty;
        println!("Mining difficulty set to: {}", difficulty);
    }
    
    // Belirli bir node'un blockchain'ini alıp karşılaştırma için kullan
    pub fn get_node_blockchain_hashes(&self, node_id: usize) -> Vec<String> {
        let mut hashes = Vec::new();
        if let Some(node) = self.nodes.get(&node_id) {
            for block in &node.blockchain {
                hashes.push(block.hash.clone());
            }
        }
        hashes
    }
} 