use rand::Rng;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// Gerekli modülleri kullan
use crate::block::Block;
use crate::wallet::Wallet;
use crate::transaction::{Transaction, UTXO, TxInput, TxOutput, get_utxo_id};

//Node sınıfı
#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub connections: Vec<usize>, // Bağlı nodeların id'leri
    pub is_validator: bool,
    pub blockchain: Vec<Block>,  // Blok zinciri
    pub wallet: Wallet,         // Cüzdan
    pub mempool: Vec<Transaction>, // Henüz bloklara eklenmemiş işlemler
    pub utxo_set: Vec<UTXO>,    // Tüm harcanmamış çıktılar (UTXO seti)
    pub mining_reward: u64,     // Madencilik ödülü
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node ID: {}, Address: {}, Validator: {}, Connections: {:?}, Balance: {} coin, Blockchain Length: {}", 
            self.id, self.wallet.get_address(), self.is_validator, self.connections, 
            self.wallet.get_balance() as f64 / 100_000_000.0, self.blockchain.len())
    }
}

impl Node {
    pub fn new(id: usize, genesis_block: Option<Block>) -> Self {
        let wallet = Wallet::new(); // Yeni bir cüzdan oluştur
        let mut blockchain = Vec::new();
        let mut utxo_set = Vec::new();
        let mut wallet_clone = wallet.clone();
        
        // Genesis bloğu dışarıdan verilmişse onu kullan
        if let Some(block) = genesis_block {
            blockchain.push(block.clone());
            
            // Genesis bloğundaki coinbase işlemini bul ve UTXO'yu ekle
            if !block.transactions.is_empty() {
                let coinbase_tx = &block.transactions[0];
                
                // Eğer bu node'un adresi ile coinbase işleminin alıcı adresi aynıysa UTXO'yu ekle
                if coinbase_tx.outputs[0].recipient_address == wallet.get_address() {
                    let genesis_utxo = UTXO {
                        transaction_id: coinbase_tx.id.clone(),
                        output_index: 0,
                        amount: coinbase_tx.outputs[0].amount,
                        recipient_address: coinbase_tx.outputs[0].recipient_address.clone(),
                    };
                    utxo_set.push(genesis_utxo.clone());
                    wallet_clone.add_utxo(genesis_utxo);
                }
            }
        }
        // Genesis bloğu verilmemişse boş bir blockchain ile başla
        // Otomatik olarak genesis bloğu oluşturmuyoruz
        
        Node {
            id,
            connections: Vec::new(),
            is_validator: false,
            blockchain,
            wallet: wallet_clone,
            mempool: Vec::new(),
            utxo_set,
            mining_reward: 50_0000_0000, // 50 coin (BTC'de olduğu gibi)
        }
    }

    // Cüzdan adresini almak için fonksiyon
    pub fn get_address(&self) -> &str {
        self.wallet.get_address()
    }
    
    // Cüzdan bakiyesini almak için fonksiyon
    pub fn get_balance(&self) -> u64 {
        self.wallet.get_balance()
    }
    
    // İşlem oluştur ve mempool'a ekle
    pub fn create_transaction(&mut self, recipient_address: &str, amount: u64) -> Option<Transaction> {
        // Cüzdanın işlem oluşturmasını iste
        if let Some(transaction) = self.wallet.create_transaction(recipient_address, amount) {
            // İşlemi doğrula
            if self.verify_transaction(&transaction) {
                // İşlemi mempool'a ekle
                self.mempool.push(transaction.clone());
                Some(transaction)
            } else {
                // Transaction doğrulanamadı
                None
            }
        } else {
            // Transaction oluşturulamadı
            None
        }
    }
    
    // İşlemi doğrula
    pub fn verify_transaction(&self, transaction: &Transaction) -> bool {
        // Coinbase işlemleri her zaman geçerlidir
        if transaction.inputs.is_empty() && !transaction.outputs.is_empty() {
            return true;
        }
        
        // İşlemin geçerli olup olmadığını kontrol et
        if !transaction.is_valid(&self.utxo_set) {
            // İşlem geçersiz - UTXO doğrulaması başarısız
            return false;
        }
        
        // Bu aşamada işlem geçerli kabul edilir
        // Gerçek bir sistemde imza doğrulaması yapılır, ancak bu simülasyonda basitleştiriyoruz
        // Çünkü her node kendi cüzdanını kullanıyor ve diğer node'ların public key'lerine erişimimiz yok
        
        // Her girdi için UTXO'nun var olduğunu kontrol et
        for input in &transaction.inputs {
            // UTXO'yu bul
            let utxo = self.utxo_set.iter().find(|utxo| {
                let utxo_id = get_utxo_id(&utxo.transaction_id, utxo.output_index);
                utxo_id == input.utxo_id
            });
            
            if utxo.is_none() {
                // İşlem geçersiz - UTXO bulunamadı
                return false;
            }
        }
        
        // Tüm kontroller geçildi, işlem geçerli
        true
    }
    
    // Mempool'dan işlemleri al ve yeni bir blok oluştur
    pub fn create_block(&mut self, difficulty: usize) -> Option<Block> {
        if !self.is_validator {
            // Node is not a validator
            return None;
        }
        
        // Mempool'dan en fazla 10 işlem al
        let mut block_transactions = Vec::new();
        let transaction_limit = 10;
        
        // Önce coinbase işlemini ekle (madencilik ödülü)
        let coinbase_tx = Transaction::new_coinbase(
            self.wallet.get_address().to_string(),
            self.mining_reward
        );
        block_transactions.push(coinbase_tx);
        
        // Mempool'dan geçerli işlemleri seç
        let mut selected_tx_indices = Vec::new();
        
        for (i, tx) in self.mempool.iter().enumerate() {
            if block_transactions.len() >= transaction_limit {
                break;
            }
            
            if self.verify_transaction(tx) {
                block_transactions.push(tx.clone());
                selected_tx_indices.push(i);
            }
        }
        
        // Seçilen işlemleri mempool'dan çıkar (büyükten küçüğe doğru silmek için)
        selected_tx_indices.sort_by(|a, b| b.cmp(a));
        for &i in &selected_tx_indices {
            self.mempool.remove(i);
        }
        
        // Yeni blok oluştur
        if let Some(last_block) = self.blockchain.last() {
            // Normal blok oluşturma (zincirde zaten en az bir blok var)
            let new_index = last_block.index + 1;
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            let mut new_block = Block::new(
                new_index,
                timestamp,
                block_transactions,
                last_block.hash.clone(),
            );
            
            // Blok madenciliği
            new_block.mine_block(difficulty);
            
            Some(new_block)
        } else {
            // Blockchain boş, genesis bloğu oluştur
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            let mut genesis_block = Block::new(
                0,
                timestamp,
                block_transactions,
                "0".to_string(), // Genesis bloğunun previous hash'i "0" olur
            );
            
            // Genesis bloğu madenciliği
            genesis_block.mine_block(difficulty);
            
            Some(genesis_block)
        }
    }
    
    // Blok içindeki işlemlere göre UTXO setini güncelle
    pub fn update_utxo_set(&mut self, block: &Block) {
        // UTXO seti güncelleniyor
        
        for tx in &block.transactions {
            // Harcanan UTXO'ları çıkar
            for input in &tx.inputs {
                if let Some(index) = self.utxo_set.iter().position(|utxo| {
                    let utxo_id = get_utxo_id(&utxo.transaction_id, utxo.output_index);
                    utxo_id == input.utxo_id
                }) {
                    let removed_utxo = &self.utxo_set[index];
                    // UTXO harcanıyor
                    self.utxo_set.remove(index);
                }
            }
            
            // Yeni UTXO'ları ekle
            for (i, output) in tx.outputs.iter().enumerate() {
                let utxo = UTXO {
                    transaction_id: tx.id.clone(),
                    output_index: i,
                    amount: output.amount,
                    recipient_address: output.recipient_address.clone(),
                };
                
                // Yeni UTXO ekleniyor
                
                self.utxo_set.push(utxo);
            }
        }
        
        // UTXO seti güncellendi
    }

    //Node bağlantısı ekleme
    pub fn add_connection(&mut self, node_id: usize) {
        if node_id == self.id {
            return;
        }
        
        if self.connections.contains(&node_id) {
            // Node zaten bağlı
            return;
        }
        
        self.connections.push(node_id);
    }
    
    // Dışarıdan gelen bir bloğu ekle
    pub fn add_block_from_network(&mut self, block: Block, difficulty: usize) -> bool {
        // Blok zincirinin geçerliliğini kontrol et
        if !self.is_valid_new_block(&block, difficulty) {
            // Geçersiz blok reddedildi
            return false;
        }
        
        // Yeni blok ekleniyor
        
        // Blockchain'e ekle
        self.blockchain.push(block.clone());
        
        // UTXO setini güncelle
        self.update_utxo_set(&block);
        
        // Cüzdanı güncelle
        self.wallet.update_utxos(&block.transactions);
        
        true
    }
    
    // Yeni bir bloğun geçerli olup olmadığını kontrol et
    pub fn is_valid_new_block(&self, block: &Block, difficulty: usize) -> bool {
        if let Some(last_block) = self.blockchain.last() {
            // Blok indeksini kontrol et
            if block.index != last_block.index + 1 {
                // Geçersiz blok indeksi
                return false;
            }
            
            // Önceki hash'i kontrol et
            if block.previous_hash != last_block.hash {
                // Geçersiz önceki hash
                return false;
            }
            
            // Hash'i kontrol et
            let calculated_hash = block.calculate_hash();
            if block.hash != calculated_hash {
                // Geçersiz blok hash'i
                return false;
            }
            
            // Proof of Work kontrolü
            let target = "0".repeat(difficulty);
            if !block.hash.starts_with(&target) {
                // Geçersiz Proof of Work
                return false;
            }
            
            // Merkle kök hash'ini kontrol et
            let calculated_merkle_root = block.calculate_merkle_root();
            if block.merkle_root != calculated_merkle_root {
                // Geçersiz merkle kök hash'i
                return false;
            }
            
            // Tüm işlemleri doğrula
            for (i, tx) in block.transactions.iter().enumerate() {
                // İlk işlem coinbase olmalı
                if i == 0 {
                    if !tx.inputs.is_empty() {
                        // Geçersiz coinbase işlemi
                        return false;
                    }
                } else if !self.verify_transaction(tx) {
                    // Geçersiz işlem
                    return false;
                }
            }
            
            true
        } else {
            // Genesis blok kontrolü
            if block.index == 0 {
                return true;
            } else {
                // Blockchain boş ama gelen blok genesis değil
                return false;
            }
        }
    }
    
    // Blockchain'i güncelle
    pub fn update_blockchain(&mut self, blockchain: Vec<Block>, difficulty: usize) {
        // Gelen blockchain'in geçerli olup olmadığını kontrol et
        if !self.is_chain_valid_with_difficulty(&blockchain, difficulty) {
            // Gelen blockchain geçerli değil
            return;
        }
        
        // Zincir uzunluğunu kontrol et (en uzun zincir kuralı)
        if blockchain.len() > self.blockchain.len() {
            // Daha uzun bir blockchain alındı
            
            // Mevcut UTXO setini temizle
            self.utxo_set.clear();
            
            // Yeni blockchain'i ayarla
            self.blockchain = blockchain.clone();
            
            // UTXO setini yeniden oluştur
            self.rebuild_utxo_set();
            
            // Cüzdanı güncelle
            let all_transactions: Vec<Transaction> = self.blockchain
                .iter()
                .flat_map(|block| block.transactions.clone())
                .collect();
            
            // Cüzdanı sıfırla ve tüm işlemleri yeniden işle
            self.wallet = Wallet::new();
            self.wallet.update_utxos(&all_transactions);
        }
    }
    
    // UTXO setini blockchain'den yeniden oluştur
    pub fn rebuild_utxo_set(&mut self) {
        self.utxo_set.clear();
        
        // Tüm blokları baştan sona işle
        for block in &self.blockchain {
            for tx in &block.transactions {
                // Harcanan UTXO'ları çıkar
                for input in &tx.inputs {
                    if let Some(index) = self.utxo_set.iter().position(|utxo| {
                        let utxo_id = get_utxo_id(&utxo.transaction_id, utxo.output_index);
                        utxo_id == input.utxo_id
                    }) {
                        self.utxo_set.remove(index);
                    }
                }
                
                // Yeni UTXO'ları ekle
                for (i, output) in tx.outputs.iter().enumerate() {
                    let utxo = UTXO {
                        transaction_id: tx.id.clone(),
                        output_index: i,
                        amount: output.amount,
                        recipient_address: output.recipient_address.clone(),
                    };
                    self.utxo_set.push(utxo);
                }
            }
        }
    }

    // Blockchain'in geçerliliğini kontrol et
    pub fn is_chain_valid(&self) -> bool {
        self.is_chain_valid_with_difficulty(&self.blockchain, 2) // Varsayılan zorluk seviyesi 2
    }
    
    // Belirli bir zorluk seviyesiyle blockchain'in geçerliliğini kontrol et
    pub fn is_chain_valid_with_difficulty(&self, chain: &[Block], difficulty: usize) -> bool {
        let target = "0".repeat(difficulty);
        
        for i in 1..chain.len() {
            let current_block = &chain[i];
            let previous_block = &chain[i - 1];
            
            // Hash doğrulaması
            if current_block.hash != current_block.calculate_hash() {
                // Geçersiz blok hash'i
                return false;
            }
            
            // Previous hash doğrulaması
            if current_block.previous_hash != previous_block.hash {
                // Geçersiz önceki hash
                return false;
            }
            
            // Proof of Work kontrolü
            if !current_block.hash.starts_with(&target) {
                // Geçersiz Proof of Work
                return false;
            }
            
            // Merkle kök hash'ini kontrol et
            if current_block.merkle_root != current_block.calculate_merkle_root() {
                // Geçersiz merkle kök hash'i
                return false;
            }
            
            // Tüm işlemleri doğrula (basitleştirilmiş, gerçek bir sistemde daha karmaşık olur)
            // İlk işlem coinbase olmalı
            if !current_block.transactions.is_empty() {
                let coinbase_tx = &current_block.transactions[0];
                if !coinbase_tx.inputs.is_empty() {
                    // Geçersiz coinbase işlemi
                    return false;
                }
            }
        }
        
        true
    }
}