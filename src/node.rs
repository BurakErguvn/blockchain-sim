use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// Gerekli modülleri kullan
use crate::block::Block;
use crate::transaction::{OutPoint, Transaction, UTXO};
use crate::wallet::Wallet;

//Node sınıfı
#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub connections: Vec<usize>, // Bağlı nodeların id'leri
    pub is_validator: bool,
    pub blockchain: Vec<Block>,            // Blok zinciri
    pub wallet: Wallet,                    // Cüzdan
    pub mempool: Vec<Transaction>,         // Henüz bloklara eklenmemiş işlemler
    pub utxo_set: HashMap<OutPoint, UTXO>, // Tüm harcanmamış çıktılar (UTXO seti)
    pub mining_reward: u64,                // Madencilik ödülü
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Node ID: {}, Address: {}, Validator: {}, Connections: {:?}, Balance: {} coin, Blockchain Length: {}",
            self.id,
            self.wallet.get_address(),
            self.is_validator,
            self.connections,
            self.wallet.get_balance() as f64 / 100_000_000.0,
            self.blockchain.len()
        )
    }
}

impl Node {
    pub fn new(id: usize, genesis_block: Option<Block>) -> Self {
        let wallet = Wallet::new(); // Yeni bir cüzdan oluştur
        let mut blockchain = Vec::new();
        let mut utxo_set = HashMap::new();
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
                        outpoint: OutPoint {
                            txid: coinbase_tx.id.clone(),
                            vout: 0,
                        },
                        amount: coinbase_tx.outputs[0].amount,
                        recipient_address: coinbase_tx.outputs[0].recipient_address.clone(),
                    };
                    utxo_set.insert(genesis_utxo.outpoint.clone(), genesis_utxo.clone());
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
    pub fn create_transaction(
        &mut self,
        recipient_address: &str,
        amount: u64,
    ) -> Option<Transaction> {
        self.create_transaction_with_fee(recipient_address, amount, 1_000)
    }

    pub fn create_transaction_with_fee(
        &mut self,
        recipient_address: &str,
        amount: u64,
        fee: u64,
    ) -> Option<Transaction> {
        // Cüzdanın işlem oluşturmasını iste
        if let Some(transaction) =
            self.wallet
                .create_transaction_with_fee(recipient_address, amount, fee)
        {
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
        self.verify_transaction_with_utxo_set(transaction, &self.utxo_set)
    }

    fn verify_transaction_with_utxo_set(
        &self,
        transaction: &Transaction,
        utxo_set: &HashMap<OutPoint, UTXO>,
    ) -> bool {
        // Coinbase işlemleri her zaman geçerlidir
        if transaction.inputs.is_empty() && !transaction.outputs.is_empty() {
            return true;
        }

        // İşlemin geçerli olup olmadığını kontrol et
        if !transaction.is_valid(utxo_set) {
            // İşlem geçersiz - UTXO doğrulaması başarısız
            return false;
        }

        // Her girdi için UTXO sahiplik ve imza kontrolü
        for (index, input) in transaction.inputs.iter().enumerate() {
            let Some(utxo) = utxo_set.get(&input.previous_output) else {
                return false;
            };

            let Some(derived_address) = Wallet::public_key_bytes_to_address(&input.public_key)
            else {
                return false;
            };
            if derived_address != utxo.recipient_address {
                return false;
            }

            let Some(signature_payload) = transaction.signing_payload(index) else {
                return false;
            };
            if !Wallet::verify_with_public_key_bytes(
                &signature_payload,
                &input.signature,
                &input.public_key,
            ) {
                return false;
            }
        }

        // Tüm kontroller geçildi, işlem geçerli
        true
    }

    fn apply_transaction_to_utxo_set(
        transaction: &Transaction,
        utxo_set: &mut HashMap<OutPoint, UTXO>,
    ) {
        for input in &transaction.inputs {
            utxo_set.remove(&input.previous_output);
        }

        for (i, output) in transaction.outputs.iter().enumerate() {
            let outpoint = OutPoint {
                txid: transaction.id.clone(),
                vout: i,
            };
            utxo_set.insert(
                outpoint.clone(),
                UTXO {
                    outpoint,
                    amount: output.amount,
                    recipient_address: output.recipient_address.clone(),
                },
            );
        }
    }

    fn reconcile_mempool_with_utxo_set(&mut self) {
        let mut working_utxo_set = self.utxo_set.clone();
        let mut reconciled = Vec::new();

        for tx in &self.mempool {
            if tx.is_coinbase() {
                continue;
            }

            if self.verify_transaction_with_utxo_set(tx, &working_utxo_set) {
                reconciled.push(tx.clone());
                Self::apply_transaction_to_utxo_set(tx, &mut working_utxo_set);
            }
        }

        self.mempool = reconciled;
    }

    // Mempool'dan işlemleri al ve yeni bir blok oluştur
    pub fn create_block(&mut self, difficulty: usize) -> Option<Block> {
        if !self.is_validator {
            // Node is not a validator
            return None;
        }

        // Mempool'dan en fazla 10 işlem al
        let mut selected_transactions = Vec::new();
        let transaction_limit: usize = 10;
        let mut total_fees = 0_u64;

        // Mempool'dan geçerli işlemleri seç
        let mut selected_tx_indices = Vec::new();
        let mut working_utxo_set = self.utxo_set.clone();
        let mut ordered_mempool = self.mempool.clone();
        ordered_mempool.sort_by(|a, b| {
            let a_rate = a.calculate_fee_rate_sat_per_kb(&self.utxo_set).unwrap_or(0);
            let b_rate = b.calculate_fee_rate_sat_per_kb(&self.utxo_set).unwrap_or(0);
            b_rate
                .cmp(&a_rate)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
                .then_with(|| a.id.cmp(&b.id))
        });

        for tx in &ordered_mempool {
            if selected_transactions.len() >= transaction_limit.saturating_sub(1) {
                break;
            }

            if self.verify_transaction_with_utxo_set(tx, &working_utxo_set) {
                let tx_fee = tx.calculate_fee(&working_utxo_set)?;
                total_fees = total_fees.checked_add(tx_fee)?;
                selected_transactions.push(tx.clone());
                if let Some(i) = self
                    .mempool
                    .iter()
                    .position(|original| original.id == tx.id)
                {
                    selected_tx_indices.push(i);
                }
                Self::apply_transaction_to_utxo_set(tx, &mut working_utxo_set);
            }
        }

        let coinbase_amount = self.mining_reward.checked_add(total_fees)?;
        let coinbase_tx =
            Transaction::new_coinbase(self.wallet.get_address().to_string(), coinbase_amount);
        let mut block_transactions = vec![coinbase_tx];
        block_transactions.extend(selected_transactions);

        // Seçilen işlemleri mempool'dan çıkar (büyükten küçüğe doğru silmek için)
        selected_tx_indices.sort_by(|a, b| b.cmp(a));
        for &i in &selected_tx_indices {
            self.mempool.remove(i);
        }

        // Yeni blok oluştur
        if let Some(last_block) = self.blockchain.last() {
            // Normal blok oluşturma (zincirde zaten en az bir blok var)
            let new_index = last_block.index + 1;
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

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
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

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
            Self::apply_transaction_to_utxo_set(tx, &mut self.utxo_set);
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
            let mut block_utxo_view = self.utxo_set.clone();
            let mut total_fees = 0_u64;
            for (i, tx) in block.transactions.iter().enumerate() {
                // İlk işlem coinbase olmalı
                if i == 0 {
                    if !tx.is_coinbase() {
                        // Geçersiz coinbase işlemi
                        return false;
                    }
                    Self::apply_transaction_to_utxo_set(tx, &mut block_utxo_view);
                } else {
                    if tx.is_coinbase() {
                        // Blok içinde tek coinbase olmalı
                        return false;
                    }
                    if !self.verify_transaction_with_utxo_set(tx, &block_utxo_view) {
                        // Geçersiz işlem
                        return false;
                    }
                    let Some(tx_fee) = tx.calculate_fee(&block_utxo_view) else {
                        return false;
                    };
                    total_fees = match total_fees.checked_add(tx_fee) {
                        Some(value) => value,
                        None => return false,
                    };
                    Self::apply_transaction_to_utxo_set(tx, &mut block_utxo_view);
                }
            }

            let Some(max_reward) = self.mining_reward.checked_add(total_fees) else {
                return false;
            };
            if block.transactions[0].get_total_output_amount() > max_reward {
                return false;
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

            // Yeni blockchain'i ayarla
            self.blockchain = blockchain.clone();

            // UTXO setini yeniden oluştur
            self.rebuild_utxo_set();

            // Cüzdan kimliğini koruyarak state'i güncelle
            self.wallet.rebuild_from_utxo_set(&self.utxo_set);

            // Yeni zincire göre mempool'u temizle
            self.reconcile_mempool_with_utxo_set();
        }
    }

    // UTXO setini blockchain'den yeniden oluştur
    pub fn rebuild_utxo_set(&mut self) {
        self.utxo_set.clear();

        // Tüm blokları baştan sona işle
        for block in &self.blockchain {
            for tx in &block.transactions {
                // Harcanan UTXO'ları çıkar
                Self::apply_transaction_to_utxo_set(tx, &mut self.utxo_set);
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
