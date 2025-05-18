use rand::Rng;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Gerekli modülleri kullan
use crate::block::Block;
use crate::node::Node;
use crate::transaction::{Transaction, UTXO};

pub struct BlockchainNetwork {
    nodes: HashMap<usize, Node>,
    current_validator_id: Option<usize>,
    difficulty: usize,         // Madencilik zorluğu
    mempool: Vec<Transaction>, // Ağ seviyesindeki işlem havuzu
    mining_reward: u64,        // Madencilik ödülü
    min_transaction_fee: u64,  // Minimum işlem ücreti
}

impl BlockchainNetwork {
    pub fn new() -> Self {
        BlockchainNetwork {
            nodes: HashMap::new(),
            current_validator_id: None,
            difficulty: 2,                // Varsayılan zorluk seviyesi
            mempool: Vec::new(),
            mining_reward: 50_0000_0000,  // 50 coin (BTC'de olduğu gibi)
            min_transaction_fee: 1000,    // Minimum işlem ücreti (0.00001 coin)
        }
    }

    //Yeni bir node ekleme
    pub fn add_node(&mut self) -> usize {
        let id = self.nodes.len();
        
        // Tüm node'ları boş blockchain ile oluştur
        // Genesis bloğu madencilik işlemi sırasında oluşturulacak
        let node = Node::new(id, None);
        self.nodes.insert(id, node);
        
        id
    }
    
    // Node'un adresini alma
    pub fn get_node_address(&self, node_id: usize) -> String {
        if let Some(node) = self.nodes.get(&node_id) {
            node.get_address().to_string()
        } else {
            "Bilinmeyen Node".to_string()
        }
    }
    
    // Yeni bir işlem oluştur
    pub fn create_transaction(&mut self, sender_id: usize, recipient_address: &str, amount: u64) -> Option<Transaction> {
        if let Some(sender_node) = self.nodes.get_mut(&sender_id) {
            // İşlemi oluştur
            if let Some(tx) = sender_node.create_transaction(recipient_address, amount) {
                // İşlemi ağ mempool'una ekle
                self.mempool.push(tx.clone());
                
                // İşlemi tüm node'lara yay
                self.broadcast_transaction(&tx);
                
                Some(tx)
            } else {
                println!("Node {} işlem oluşturamadı.", sender_id);
                None
            }
        } else {
            println!("Node {} bulunamadı.", sender_id);
            None
        }
    }
    
    // İşlemi tüm node'lara yay
    pub fn broadcast_transaction(&mut self, transaction: &Transaction) {
        // Gönderici node'un adresini al
        let sender_address = transaction.inputs[0].sender_address.clone();
        
        for (_, node) in self.nodes.iter_mut() {
            // Eğer bu node işlemin göndericisi değilse işlemi doğrula ve mempool'a ekle
            // Gönderici node zaten işlemi kendi mempool'una eklemiş olacak
            if node.wallet.get_address() != sender_address {
                // İşlem doğrulanıyorsa mempool'a ekle
                if node.verify_transaction(transaction) {
                    node.mempool.push(transaction.clone());
                }
            }
        }
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

    // Madencilik yaparak yeni bir blok oluştur
    pub fn mine_block(&mut self) -> Option<Block> {
        if let Some(validator_id) = self.current_validator_id {
            let validator = match self.nodes.get_mut(&validator_id) {
                Some(v) => v,
                None => {
                    // Validator bulunamadı
                    return None;
                }
            };
            
            // Mempool'dan işlemleri al ve yeni bir blok oluştur
            // Önce ağ mempool'undan validator'un mempool'una işlemleri aktar
            for tx in &self.mempool {
                if validator.verify_transaction(tx) {
                    validator.mempool.push(tx.clone());
                }
            }
            
            // Validator'un madencilik yapmasını iste
            let new_block = validator.create_block(self.difficulty);
            
            if let Some(block) = &new_block {
                // Validator yeni bir blok oluşturdu
                
                // İşlemleri ağ mempool'undan çıkar
                self.mempool.retain(|tx| {
                    !block.transactions.iter().any(|block_tx| block_tx.id == tx.id)
                });
                
                // Önce validator'un kendi blockchain'ine bloğu ekle
                if let Some(validator) = self.nodes.get_mut(&validator_id) {
                    validator.blockchain.push(block.clone());
                    validator.update_utxo_set(block);
                    validator.wallet.update_utxos(&block.transactions);
                }
                
                // Yeni bloğu tüm node'lara yay
                self.broadcast_block(block);
            }
            
            new_block
        } else {
            // Seçili validator yok
            None
        }
    }

    //Hash'i tüm bağlı node'lara gönder
    pub fn broadcast_hash(&mut self, hash: String){
        for node in self.nodes.values_mut()  {
            // Node'un hash'i yok, bu satırı kaldırıyoruz
        }
        println!("Broadcasted hash {} to all nodes.", hash);
    }
    
    // Yeni bir bloğu tüm node'lara yay
    pub fn broadcast_block(&mut self, block: &Block) {
        for (id, node) in self.nodes.iter_mut() {
            if let Some(validator_id) = self.current_validator_id {
                if *id != validator_id { // Validator dışındaki tüm node'lara
                    let result = node.add_block_from_network(block.clone(), self.difficulty);
                    if result {
                        println!("Node {}: Yeni blok kabul edildi", id);
                    }
                }
            } else {
                // Validator yoksa tüm node'lara yayınla
                let result = node.add_block_from_network(block.clone(), self.difficulty);
                if result {
                    println!("Node {}: Güncel bakiye: {} coin", id, node.get_balance() as f64 / 100_000_000.0);
                    println!("Node {}: Yeni blok kabul edildi", id);
                } else {
                    println!("Node {}: Blok reddedildi", id);
                }
            }
        }
    }

    // Blockchain'i tüm node'lara yayınla
    pub fn broadcast_blockchain(&mut self, blockchain: Vec<Block>) {
        for (id, node) in self.nodes.iter_mut() {
            if let Some(validator_id) = self.current_validator_id {
                if *id != validator_id { // Validator dışındaki tüm node'lara
                    node.update_blockchain(blockchain.clone(), self.difficulty);
                }
            } else {
                // Validator yoksa tüm node'lara yayınla
                node.update_blockchain(blockchain.clone(), self.difficulty);
            }
        }
    }

    //Bir node'un hash'ini manipüle etmeyi dene
    pub fn try_manipulate_hash(&mut self, node_id : usize, fake_hash: String) -> bool {
        if let Some(validator_id) = self.current_validator_id {
            if node_id == validator_id {
                //Eğer validator hash'i değiştirirse, bu yeni hash olur
                if let Some(node) = self.nodes.get_mut(&node_id){
                    // Node'un hash'i yok, bu satırı kaldırıyoruz
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
                    if id != &node_id { // Node'un hash'i yok, sadece ID'ye göre kontrol ediyoruz
                        matching_hash_count += 1;
                    }
                    
                }

                //Konsensüs kontrolü
                if matching_hash_count > total_nodes / 2 {
                    //Konsensüs sağlandı, hash düzeltilecek
                    if let Some(node) = self.nodes.get_mut(&node_id) {
                        // Node'un hash'i yok, bu satırı kaldırıyoruz
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
                // Artık data yerine transactions kullanıyoruz
                // Manipülasyon için işlemleri değiştirebiliriz
                // Örneğin: İlk işlemin ID'sini değiştirelim
                if !last_block.transactions.is_empty() {
                    last_block.transactions[0].id = format!("Manipulated: {}", last_block.transactions[0].id);
                }
                
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
                    node.update_blockchain(blockchain.clone(), self.difficulty);
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
        for (id, node) in &self.nodes {
            println!("Node {}: {} coin, Blockchain Length: {}", 
                id, node.get_balance() as f64 / 100_000_000.0, node.blockchain.len());
        }
        println!("---------------------------------\n");
    }
    
    // Belirli bir node'un blockchain'ini görüntüle
    pub fn print_blockchain(&self, node_id: usize) {
        if let Some(node) = self.nodes.get(&node_id) {
            println!("\n--- BLOCKCHAIN FROM NODE {} ---", node_id);
            for (i, block) in node.blockchain.iter().enumerate() {
                println!("Blok {}: Hash: {}, İşlem Sayısı: {}", 
                    i, block.hash, block.transactions.len());
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