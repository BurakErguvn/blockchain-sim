use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Gerekli modülleri kullan
use crate::block::Block;
use crate::node::Node;
use crate::transaction::{OutPoint, Transaction, UTXO};

#[derive(Clone, Debug)]
struct MempoolTxInfo {
    size_bytes: usize,
    fee_sat: u64,
    fee_rate_sat_per_kb: u64,
    arrival_sequence: u64,
}

pub struct BlockchainNetwork {
    pub nodes: Vec<Node>,
    pub mempool: Vec<Transaction>,
    pub mempool_spent_outpoints: HashMap<OutPoint, String>,
    mempool_policy: HashMap<String, MempoolTxInfo>,
    mempool_sequence_counter: u64,
    mempool_total_bytes: usize,
    pub max_mempool_bytes: usize,
    pub min_fee_rate_sat_per_kb: u64,
    pub replacement_increment_sat_per_kb: u64,
    pub current_validator_id: Option<usize>,
    pub difficulty: usize,
    pub block_time: u64,      // Saniye cinsinden blok oluşturma süresi
    pub last_block_time: u64, // Son bloğun oluşturulduğu zaman
    pub mining_active: bool,  // Madencilik aktif mi?
    pub mining_thread: Option<thread::JoinHandle<()>>, // Madencilik thread'i
    pub stop_sender: Option<mpsc::Sender<bool>>, // Madencilik durdurma sinyali
}

impl BlockchainNetwork {
    const DEFAULT_MAX_MEMPOOL_BYTES: usize = 300 * 1024 * 1024;
    const DEFAULT_MIN_FEE_RATE_SAT_PER_KB: u64 = 1_000;
    const DEFAULT_REPLACEMENT_INCREMENT_SAT_PER_KB: u64 = 100;

    pub fn new() -> Self {
        // Şu anki zamanı al
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Zaman alınamadı")
            .as_secs();

        BlockchainNetwork {
            nodes: Vec::new(),
            mempool: Vec::new(),
            mempool_spent_outpoints: HashMap::new(),
            mempool_policy: HashMap::new(),
            mempool_sequence_counter: 0,
            mempool_total_bytes: 0,
            max_mempool_bytes: Self::DEFAULT_MAX_MEMPOOL_BYTES,
            min_fee_rate_sat_per_kb: Self::DEFAULT_MIN_FEE_RATE_SAT_PER_KB,
            replacement_increment_sat_per_kb: Self::DEFAULT_REPLACEMENT_INCREMENT_SAT_PER_KB,
            current_validator_id: None,
            difficulty: 2,        // Varsayılan zorluk seviyesi
            block_time: 10,       // Varsayılan olarak 10 saniye
            last_block_time: now, // Başlangıç zamanı
            mining_active: false,
            mining_thread: None,
            stop_sender: None,
        }
    }

    // Otomatik madencilik işlemini başlat
    pub fn start_automatic_mining(&mut self) -> Result<(), String> {
        if self.mining_active {
            return Err("Madencilik zaten aktif".to_string());
        }

        // Önce bir validator seçilmiş olmalı
        if self.current_validator_id.is_none() {
            return Err("Madencilik başlamadan önce bir validator seçilmelidir".to_string());
        }

        // Durdurma sinyali için kanal oluştur
        let (stop_sender, stop_receiver) = mpsc::channel();
        self.stop_sender = Some(stop_sender);

        // Thread için gerekli bilgileri kopyala
        let block_time = self.block_time;
        let validator_id = self.current_validator_id;
        let difficulty = self.difficulty;

        // Thread'de kullanmak için network'un bir kopyasını oluştur
        let nodes_clone = self.nodes.clone();

        // Madencilik thread'ini başlat
        let mining_thread = thread::spawn(move || {
            let mut last_mine_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Zaman alınamadı")
                .as_secs();

            loop {
                // Durdurma sinyali geldi mi kontrol et
                if let Ok(_) = stop_receiver.try_recv() {
                    println!("Madencilik durduruldu");
                    break;
                }

                // Şu anki zamanı al
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Zaman alınamadı")
                    .as_secs();

                // Son madencilikten beri geçen süreyi kontrol et
                if now - last_mine_time >= block_time {
                    // Son madencilik zamanını güncelle
                    last_mine_time = now;

                    // 1 saniye bekle - madencilik işleminin tamamlanmasını simule et
                    thread::sleep(Duration::from_secs(1));
                } else {
                    // Kısa bir süre bekle
                    thread::sleep(Duration::from_millis(1000));
                }
            }
        });

        self.mining_thread = Some(mining_thread);
        self.mining_active = true;

        Ok(())
    }

    // Otomatik madencilik işlemini durdur
    pub fn stop_automatic_mining(&mut self) -> Result<(), String> {
        if !self.mining_active {
            return Err("Madencilik zaten durdurulmuş".to_string());
        }

        // Durdurma sinyali gönder
        if let Some(sender) = &self.stop_sender {
            if let Err(_) = sender.send(true) {
                return Err("Madencilik thread'ine sinyal gönderilemedi".to_string());
            }
        } else {
            return Err("Durdurma sinyali gönderici bulunamadı".to_string());
        }

        // Thread'in tamamlanmasını bekle
        if let Some(thread) = self.mining_thread.take() {
            if let Err(_) = thread.join() {
                return Err("Madencilik thread'i beklenirken hata oluştu".to_string());
            }
        }

        self.mining_active = false;
        self.stop_sender = None;

        Ok(())
    }

    // Not: Clone trait'i artık kullanmıyoruz, çünkü madencilik thread'i doğrudan ana network'e referans veriyor

    // Yeni bir node ekleme
    pub fn add_node(&mut self) -> usize {
        let id = self.nodes.len();

        // Tüm node'ları boş blockchain ile oluştur
        // Genesis bloğu madencilik işlemi sırasında oluşturulacak
        let node = Node::new(id, None);
        self.nodes.push(node);

        id
    }

    // Node'un adresini alma
    pub fn get_node_address(&self, node_id: usize) -> String {
        if let Some(node) = self.nodes.get(node_id) {
            node.get_address().to_string()
        } else {
            "Bilinmeyen Node".to_string()
        }
    }

    // Yeni bir işlem oluştur
    pub fn create_transaction(
        &mut self,
        sender_id: usize,
        recipient_address: &str,
        amount: u64,
    ) -> Option<Transaction> {
        self.create_transaction_with_fee(sender_id, recipient_address, amount, 1_000)
    }

    pub fn create_transaction_with_fee(
        &mut self,
        sender_id: usize,
        recipient_address: &str,
        amount: u64,
        fee: u64,
    ) -> Option<Transaction> {
        if sender_id >= self.nodes.len() {
            None
        } else {
            let (tx, sender_utxo_snapshot) = {
                let sender_node = self.nodes.get_mut(sender_id)?;
                let tx = sender_node.create_transaction_with_fee(recipient_address, amount, fee)?;
                (tx, sender_node.utxo_set.clone())
            };

            let tx_info =
                Self::calculate_tx_info(&tx, &sender_utxo_snapshot, self.next_mempool_sequence())?;
            if tx_info.fee_rate_sat_per_kb < self.min_fee_rate_sat_per_kb {
                if let Some(sender_node) = self.nodes.get_mut(sender_id) {
                    sender_node
                        .mempool
                        .retain(|existing_tx| existing_tx.id != tx.id);
                }
                return None;
            }

            let conflicting_tx_ids = self.collect_conflicting_tx_ids(&tx);
            if !conflicting_tx_ids.is_empty() {
                if !self.can_replace_conflicts(tx_info.fee_rate_sat_per_kb, &conflicting_tx_ids) {
                    if let Some(sender_node) = self.nodes.get_mut(sender_id) {
                        sender_node
                            .mempool
                            .retain(|existing_tx| existing_tx.id != tx.id);
                    }
                    return None;
                }

                for conflict_id in conflicting_tx_ids {
                    self.remove_transaction_from_mempool(&conflict_id);
                }
            }

            self.add_transaction_to_mempool(tx.clone(), tx_info);
            self.trim_mempool_to_limit();

            if self.mempool_policy.get(&tx.id).is_none() {
                if let Some(sender_node) = self.nodes.get_mut(sender_id) {
                    sender_node
                        .mempool
                        .retain(|existing_tx| existing_tx.id != tx.id);
                }
                return None;
            }

            // İşlemi tüm node'lara yay
            self.broadcast_transaction(&tx);

            Some(tx)
        }
    }

    fn next_mempool_sequence(&mut self) -> u64 {
        self.mempool_sequence_counter = self.mempool_sequence_counter.saturating_add(1);
        self.mempool_sequence_counter
    }

    fn calculate_tx_info(
        transaction: &Transaction,
        utxo_set: &HashMap<OutPoint, UTXO>,
        arrival_sequence: u64,
    ) -> Option<MempoolTxInfo> {
        let fee_sat = transaction.calculate_fee(utxo_set)?;
        let size_bytes = transaction.estimated_size_bytes();
        let fee_rate_sat_per_kb = transaction.calculate_fee_rate_sat_per_kb(utxo_set)?;

        Some(MempoolTxInfo {
            size_bytes,
            fee_sat,
            fee_rate_sat_per_kb,
            arrival_sequence,
        })
    }

    fn collect_conflicting_tx_ids(&self, transaction: &Transaction) -> HashSet<String> {
        let mut conflicts = HashSet::new();
        for input in &transaction.inputs {
            if let Some(tx_id) = self.mempool_spent_outpoints.get(&input.previous_output) {
                conflicts.insert(tx_id.clone());
            }
        }
        conflicts
    }

    fn can_replace_conflicts(
        &self,
        new_fee_rate_sat_per_kb: u64,
        conflicting_tx_ids: &HashSet<String>,
    ) -> bool {
        conflicting_tx_ids.iter().all(|tx_id| {
            self.mempool_policy.get(tx_id).is_some_and(|existing| {
                new_fee_rate_sat_per_kb
                    >= existing
                        .fee_rate_sat_per_kb
                        .saturating_add(self.replacement_increment_sat_per_kb)
            })
        })
    }

    fn add_transaction_to_mempool(&mut self, transaction: Transaction, tx_info: MempoolTxInfo) {
        for input in &transaction.inputs {
            self.mempool_spent_outpoints
                .insert(input.previous_output.clone(), transaction.id.clone());
        }

        self.mempool_total_bytes = self.mempool_total_bytes.saturating_add(tx_info.size_bytes);
        self.mempool_policy.insert(transaction.id.clone(), tx_info);
        self.mempool.push(transaction);
    }

    fn remove_transaction_from_nodes_mempool(&mut self, tx_id: &str) {
        for node in &mut self.nodes {
            node.mempool.retain(|tx| tx.id != tx_id);
        }
    }

    fn remove_transaction_from_mempool(&mut self, tx_id: &str) {
        if let Some(position) = self.mempool.iter().position(|tx| tx.id == tx_id) {
            let removed = self.mempool.remove(position);
            for input in &removed.inputs {
                self.mempool_spent_outpoints.remove(&input.previous_output);
            }
            if let Some(info) = self.mempool_policy.remove(tx_id) {
                self.mempool_total_bytes = self.mempool_total_bytes.saturating_sub(info.size_bytes);
            }
            self.remove_transaction_from_nodes_mempool(tx_id);
        }
    }

    fn trim_mempool_to_limit(&mut self) {
        while self.mempool_total_bytes > self.max_mempool_bytes {
            let candidate = self
                .mempool
                .iter()
                .filter_map(|tx| {
                    self.mempool_policy.get(&tx.id).map(|info| {
                        (
                            tx.id.clone(),
                            info.fee_rate_sat_per_kb,
                            info.arrival_sequence,
                            info.fee_sat,
                        )
                    })
                })
                .min_by(|a, b| {
                    a.1.cmp(&b.1)
                        .then_with(|| a.2.cmp(&b.2))
                        .then_with(|| a.3.cmp(&b.3))
                });

            let Some((tx_id, _, _, _)) = candidate else {
                break;
            };
            self.remove_transaction_from_mempool(&tx_id);
        }
    }

    fn rebuild_mempool_outpoint_index(&mut self) {
        self.mempool_spent_outpoints.clear();
        self.mempool_total_bytes = 0;
        for tx in &self.mempool {
            for input in &tx.inputs {
                self.mempool_spent_outpoints
                    .insert(input.previous_output.clone(), tx.id.clone());
            }
            if let Some(info) = self.mempool_policy.get(&tx.id) {
                self.mempool_total_bytes = self.mempool_total_bytes.saturating_add(info.size_bytes);
            }
        }
    }

    fn rebuild_mempool_policy_from_utxo_set(&mut self, utxo_set: &HashMap<OutPoint, UTXO>) {
        self.mempool_policy.clear();
        self.mempool_total_bytes = 0;
        let mut valid_mempool = Vec::new();

        let mempool_snapshot = self.mempool.clone();
        for tx in mempool_snapshot {
            if tx.is_coinbase() {
                continue;
            }
            if let Some(info) = Self::calculate_tx_info(&tx, utxo_set, self.next_mempool_sequence())
            {
                self.mempool_total_bytes = self.mempool_total_bytes.saturating_add(info.size_bytes);
                self.mempool_policy.insert(tx.id.clone(), info);
                valid_mempool.push(tx);
            }
        }

        self.mempool = valid_mempool;
        self.rebuild_mempool_outpoint_index();
    }

    fn reconcile_network_mempool_after_chain_sync(&mut self) {
        let Some(reference_node) = self.nodes.first() else {
            self.mempool.clear();
            self.mempool_policy.clear();
            self.mempool_spent_outpoints.clear();
            self.mempool_total_bytes = 0;
            return;
        };

        let reference_utxo_set = reference_node.utxo_set.clone();
        self.mempool
            .retain(|tx| !tx.is_coinbase() && reference_node.verify_transaction(tx));
        self.rebuild_mempool_policy_from_utxo_set(&reference_utxo_set);
        self.trim_mempool_to_limit();
    }
    // İşlemi tüm node'lara yay
    pub fn broadcast_transaction(&mut self, transaction: &Transaction) {
        // Gönderici node'un adresini al (coinbase işlemlerinde gönderici olmaz)
        let sender_address = transaction
            .inputs
            .first()
            .map(|input| input.sender_address.clone());

        for node in self.nodes.iter_mut() {
            // Eğer bu node işlemin göndericisi değilse işlemi doğrula ve mempool'a ekle
            // Gönderici node zaten işlemi kendi mempool'una eklemiş olacak
            if sender_address
                .as_ref()
                .is_some_and(|address| node.wallet.get_address() == address)
            {
                continue;
            }

            if node.verify_transaction(transaction) {
                node.mempool.push(transaction.clone());
            }
        }
    }

    // İki node arasında bağlantı oluşturma
    pub fn connect_nodes(&mut self, node1_id: usize, node2_id: usize) {
        if node1_id == node2_id {
            println!("Warning: Cannot connect a node to itself.");
            return;
        }

        if let Some(node1) = self.nodes.get_mut(node1_id) {
            node1.add_connection(node2_id);
        } else {
            println!("Warning: Node {} not found.", node1_id);
        }

        if let Some(node2) = self.nodes.get_mut(node2_id) {
            node2.add_connection(node1_id);
        } else {
            println!("Warning: Node {} not found.", node2_id);
        }
    }

    // Rasgele bir validator seç
    pub fn select_random_validator(&mut self) {
        if self.nodes.is_empty() {
            println!("Warning: No nodes available to select as validator.");
            return;
        }

        // Önce tüm node'ları validator olmaktan çıkar
        for node in self.nodes.iter_mut() {
            node.is_validator = false;
        }

        // Rasgele bir node seç
        let mut rng = rand::thread_rng();
        let validator_id = rng.gen_range(0..self.nodes.len());

        if let Some(node) = self.nodes.get_mut(validator_id) {
            node.is_validator = true;
            self.current_validator_id = Some(validator_id);
            println!("Node {} is selected as the new validator.", validator_id);
        }
    }

    // Madencilik yaparak yeni bir blok oluştur
    pub fn mine_block(&mut self) -> Option<Block> {
        // Şu anki zamanı al
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Zaman alınamadı")
            .as_secs();

        // Son blok zamanını güncelle
        self.last_block_time = now;

        if let Some(validator_id) = self.current_validator_id {
            let mut removed_tx_ids_after_mining: Vec<String> = Vec::new();

            // Validator'u al
            let validator = match self.nodes.get_mut(validator_id) {
                Some(v) => v,
                None => {
                    println!("Validator bulunamadı!");
                    return None;
                }
            };

            // Mempool'dan işlemleri al ve yeni bir blok oluştur
            // Önce ağ mempool'undan validator'un mempool'una işlemleri aktar
            let mempool_copy = self.mempool.clone();
            for tx in &mempool_copy {
                if validator.verify_transaction(tx) {
                    validator.mempool.push(tx.clone());
                }
            }

            // Validator'un madencilik yapmasını iste
            let new_block = validator.create_block(self.difficulty);

            if let Some(block) = &new_block {
                // Son blok zamanını güncelle
                self.last_block_time = now;

                let removed_tx_ids: HashSet<String> =
                    block.transactions.iter().map(|tx| tx.id.clone()).collect();

                // İşlemleri ağ mempool'undan çıkar
                self.mempool.retain(|tx| {
                    !block
                        .transactions
                        .iter()
                        .any(|block_tx| block_tx.id == tx.id)
                });

                for tx_id in &removed_tx_ids {
                    self.mempool_policy.remove(tx_id);
                }
                removed_tx_ids_after_mining = removed_tx_ids.into_iter().collect();

                // Validator'un blockchain'ine bloğu ekle
                validator.blockchain.push(block.clone());
                validator.update_utxo_set(block);
                validator.wallet.update_utxos(&block.transactions);
                self.rebuild_mempool_outpoint_index();

                // Yeni bloğu tüm node'lara yay
                self.broadcast_block(block);

                // Yeni bir validator seç
                self.select_random_validator();
            } else {
                println!("Blok oluşturulamadı!");
            }

            for tx_id in &removed_tx_ids_after_mining {
                self.remove_transaction_from_nodes_mempool(tx_id);
            }

            new_block
        } else {
            println!("Madencilik için seçili validator yok!");
            None
        }
    }

    // Hash'i tüm bağlı node'lara gönder
    pub fn broadcast_hash(&mut self, hash: String) {
        for _node in self.nodes.iter_mut() {
            // Node'un hash'i yok, bu satırı kaldırıyoruz
        }
        println!("Broadcasted hash {} to all nodes.", hash);
    }

    // Yeni bir bloğu tüm node'lara yay
    pub fn broadcast_block(&mut self, block: &Block) {
        for (id, node) in self.nodes.iter_mut().enumerate() {
            if let Some(validator_id) = self.current_validator_id {
                if id != validator_id {
                    // Validator dışındaki tüm node'lara
                    let _ = node.add_block_from_network(block.clone(), self.difficulty);
                }
            } else {
                // Validator seçilmemişse tüm node'lara gönder
                let _ = node.add_block_from_network(block.clone(), self.difficulty);
            }
        }
    }

    // Blockchain'i tüm node'lara yayınla
    pub fn broadcast_blockchain(&mut self, blockchain: Vec<Block>) {
        for (id, node) in self.nodes.iter_mut().enumerate() {
            if let Some(validator_id) = self.current_validator_id {
                if id != validator_id {
                    // Validator dışındaki tüm node'lara
                    node.update_blockchain(blockchain.clone(), self.difficulty);
                }
            } else {
                // Validator seçilmemişse tüm node'lara gönder
                node.update_blockchain(blockchain.clone(), self.difficulty);
            }
        }
        self.reconcile_network_mempool_after_chain_sync();
    }

    // Bir node'un hash'ini manipüle etmeyi dene
    pub fn try_manipulate_hash(&mut self, node_id: usize, fake_hash: String) -> bool {
        if let Some(validator_id) = self.current_validator_id {
            if node_id == validator_id {
                // Eğer validator hash'i değiştirirse, bu yeni hash olur
                if let Some(_node) = self.nodes.get_mut(node_id) {
                    // Node'un hash'i yok, bu satırı kaldırıyoruz
                    self.broadcast_hash(fake_hash.clone());
                    println!("Validator has changed the hash. New hash: {}", fake_hash);
                    return true;
                }
            }
        } else {
            // Validator olmayan bir node hash'i değiştirmeye çalışırsa
            if let Some(_node) = self.nodes.get_mut(node_id) {
                let orginal_hash = fake_hash.clone();
                println!(
                    "Node {} tried to manipulate the hash: {} -> {}",
                    node_id, orginal_hash, fake_hash
                );

                // Oylama yap - %51 konsensüs gerekli
                let total_nodes = self.nodes.len();
                let mut matching_hash_count = 0;
                for (id, _other_node) in self.nodes.iter().enumerate() {
                    if id != node_id {
                        // Node'un hash'i yok, sadece ID'ye göre kontrol ediyoruz
                        matching_hash_count += 1;
                    }
                }

                // Konsensüs kontrolü
                if matching_hash_count > total_nodes / 2 {
                    // Konsensüs sağlandı, hash düzeltilecek
                    if let Some(_node) = self.nodes.get_mut(node_id) {
                        // Node'un hash'i yok, bu satırı kaldırıyoruz
                        println!(
                            "Consensus achieved! Fixed hash {} of node {}",
                            orginal_hash, node_id
                        );
                        return false;
                    }
                } else {
                    // Konsensüs sağlanamadı, manipülasyon başarılı
                    println!(
                        "Consensus failed! Node {}'s hash manipulation was successful",
                        node_id
                    );
                    return true;
                }
            }
        }
        false
    }

    // Bir node'un blockchain'ini manipüle etmeyi dene
    pub fn try_manipulate_blockchain(
        &mut self,
        node_id: usize,
        custom_hash: Option<String>,
    ) -> bool {
        let mut manipulation_successful = false;
        let mut manipulated_chain_is_valid = true;
        let mut last_block_index = 0;

        // Önce node'un blockchain'ini al
        let difficulty = self.difficulty; // Zorluk seviyesini al

        {
            let node = match self.nodes.get_mut(node_id) {
                Some(n) => n,
                None => return false,
            };

            if node.blockchain.is_empty() {
                println!(
                    "Node {}'s blockchain is empty, nothing to manipulate.",
                    node_id
                );
                return false;
            }

            // Son bloğu al
            let last_block = node.blockchain.last().unwrap();
            last_block_index = last_block.index;

            // Eğer özel bir hash verilmişse, son bloğun hash'ini değiştir
            if let Some(hash) = custom_hash {
                println!("Attempting to manipulate Node {}'s blockchain by changing the last block hash to: {}", node_id, hash);

                // Yeni bir blok oluştur ve son bloğun yerine koy
                let mut manipulated_block = last_block.clone();
                manipulated_block.hash = hash;

                // Son bloğu değiştir
                node.blockchain.pop();
                node.blockchain.push(manipulated_block);

                // Bu durumda zincir geçersiz olacak
                manipulated_chain_is_valid = false;
            } else {
                // Özel hash verilmemişse, son bloğun içeriğini değiştir ama hash'i yeniden hesapla
                println!("Attempting to manipulate Node {}'s blockchain by changing the last block content and recalculating hash.", node_id);

                // Yeni bir blok oluştur
                let mut manipulated_block = last_block.clone();

                // Bloğun timestamp'ini değiştir
                manipulated_block.timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                // Hash'i yeniden hesapla
                manipulated_block.hash = manipulated_block.calculate_hash();

                // Proof of Work yap (geçerli bir hash oluştur)
                manipulated_block.mine_block(difficulty);

                // Son bloğu değiştir
                node.blockchain.pop();
                node.blockchain.push(manipulated_block);

                // Bu durumda zincir geçerli olacak (PoW yapıldığı için)
                manipulated_chain_is_valid = true;
            }

            // Zincirin geçerliliğini kontrol et
            if !node.is_chain_valid_with_difficulty(&node.blockchain, difficulty) {
                println!("Node {}'s manipulated blockchain is invalid.", node_id);
                manipulated_chain_is_valid = false;
            } else {
                println!(
                    "Node {}'s manipulated blockchain is still valid (has valid PoW).",
                    node_id
                );
                manipulated_chain_is_valid = true;
            }
        }

        // Diğer node'ların geçerlilik durumunu kontrol et ve geçerli blockchain'leri topla
        let mut valid_chains_count = 0;
        let total_nodes = self.nodes.len();
        let mut valid_blockchain_source = None;

        for (id, node) in self.nodes.iter().enumerate() {
            if id != node_id && node.is_chain_valid() {
                valid_chains_count += 1;

                if valid_blockchain_source.is_none() {
                    valid_blockchain_source = Some((id, node.blockchain.clone()));
                }
            }
        }

        // Eğer geçerli zincirler çoğunluktaysa manipülasyon başarısız olur (konsensüs mekanizması)
        if valid_chains_count > (total_nodes / 2) {
            println!("Manipulation detected! Despite valid PoW, Node {}'s blockchain will be rejected by consensus.", node_id);

            // Geçerli blockchain'i al
            let valid_blockchain = if let Some((source_id, blockchain)) = valid_blockchain_source {
                // Geçerli bir zinciri manipüle edilen node'a gönder
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.update_blockchain(blockchain.clone(), self.difficulty);
                    println!(
                        "Node {}'s blockchain restored from Node {}.",
                        node_id, source_id
                    );
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

    // Ağın durumunu görüntüle
    pub fn print_network_state(&self) {
        println!("\n--- BLOCKCHAIN NETWORK STATE ---");
        for (id, node) in self.nodes.iter().enumerate() {
            // Doğrudan node'un wallet'inden bakiyeyi al
            let balance = node.wallet.get_balance();
            let blockchain_len = node.blockchain.len();
            let is_validator = if Some(id) == self.current_validator_id {
                "(Validator)"
            } else {
                ""
            };
            println!(
                "Node {}{}: {} coin, Blockchain Length: {}",
                id,
                is_validator,
                balance as f64 / 100_000_000.0,
                blockchain_len
            );
        }
        println!("---------------------------------\n");
    }

    // Belirli bir node'un blockchain'ini görüntüle
    pub fn print_blockchain(&self, node_id: usize) {
        if let Some(node) = self.nodes.get(node_id) {
            println!("\n--- BLOCKCHAIN FROM NODE {} ---", node_id);
            for (i, block) in node.blockchain.iter().enumerate() {
                println!(
                    "Blok {}: Hash: {}, İşlem Sayısı: {}",
                    i,
                    block.hash,
                    block.transactions.len()
                );
            }
            println!("---------------------------------\n");
        }
    }

    // Ağdaki node sayısını döndür
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    // Şu anki validator id'sini döndür
    pub fn current_val_id(&self) -> Option<usize> {
        self.current_validator_id
    }

    // Zorluk seviyesini ayarla
    pub fn set_difficulty(&mut self, difficulty: usize) {
        self.difficulty = difficulty;
        println!("Mining difficulty set to: {}", difficulty);
    }

    // Block time'ı ayarla (saniye cinsinden)
    pub fn set_block_time(&mut self, seconds: u64) {
        self.block_time = seconds;
    }

    // Belirli bir node'un blockchain'ini alıp karşılaştırma için kullan
    pub fn get_node_blockchain_hashes(&self, node_id: usize) -> Vec<String> {
        let mut hashes = Vec::new();
        if let Some(node) = self.nodes.get(node_id) {
            for block in &node.blockchain {
                hashes.push(block.hash.clone());
            }
        }
        hashes
    }
}
