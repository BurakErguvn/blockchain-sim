use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;
use rand;
use rand::Rng;
use rand::thread_rng;
use sha2::{Sha256, Digest};

mod node;
mod network;
mod block;

use network::BlockchainNetwork;

fn main() {
    let mut network = BlockchainNetwork::new();

    // Madencilik zorluğunu ayarla (2 = hash'in başında 2 tane 0 olmalı)
    network.set_difficulty(2);

    // 10 tane node oluştur
    for _ in 0..10 {
        network.add_node();
    }

    // Node'ları birbirine bağla (tam bağlı ağ)
    println!("\nCreating connections between nodes...");
    for i in 0..network.node_count() {
        for j in (i + 1)..network.node_count() {
            network.connect_nodes(i, j);
        }
    }
    println!("All connections established.");

    //Başlangıç durumunu görüntüle
    println!("Blockchain Network Created.");
    network.print_network_state();

    //Rasgele bir validator seç
    network.select_random_validator();

    // İlk 5 transaction'ı oluştur
    println!("\n--- İLK 5 TRANSACTION OLUŞTURULUYOR ---");
    for i in 1..=5 {
        // Yeni bir validator seç (her transaction için farklı validator)
        network.select_random_validator();
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let transaction = format!("Tx {}-{}-{}", format!("Transaction_{}", i), timestamp, i);
        println!("\nTransaction #{}: {}", i, transaction);
        
        // İşlemi ağa gönder
        network.create_transaction(&transaction);
        
        // Kısa bir bekleyiş ekle
        thread::sleep(Duration::from_millis(500));
    }
    
    // İlk node'un blockchain'ini görüntüle
    if network.node_count() > 0 {
        println!("\n--- BLOCKCHAIN DURUMU (5 TRANSACTION SONRASI) ---");
        network.print_blockchain(0);
    }
    
    // Ağın durumunu görüntüle
    network.print_network_state();

    //Biraz bekleyelim
    thread::sleep(Duration::from_secs(2));
    
    // Bir node'un blockchain'ini manipüle etmeyi dene
    println!("\n--- NODE BLOCKCHAIN MANIPULATION TEST ---");
    
    // Saldırgan node'u seç
    let attacker_id = rand::thread_rng().gen_range(0..network.node_count());
    
    // Sahte transaction hash'i oluştur
    let fake_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let fake_transaction = format!("Tx {}-{}", "Fake_transaction", fake_timestamp);
    
    let mut hasher = Sha256::new();
    hasher.update(fake_transaction.as_bytes());
    let fake_hash_bytes = hasher.finalize();
    let fake_hash = format!("{:x}", fake_hash_bytes); // Normal hash - PoW olmadan
    
    println!("Node {} is trying to manipulate the blockchain.", attacker_id);
    println!("Sahte işlem içeriği: {}", fake_transaction);
    println!("Oluşturulan sahte hash: {}", fake_hash);
    println!("Not: Bu hash zorluk seviyesine uygun değil (PoW yok). try_manipulate_blockchain içinde madencilik yapılacak.");
    network.try_manipulate_blockchain(attacker_id, Some(fake_hash)); // Sahte transaction hash'i ile manipülasyon
    
    // Başka bir node'un (sağlam) blockchain'ini görüntüle
    let honest_node_id = (attacker_id + 1) % network.node_count();
    
    // Network durumunu göster
    network.print_network_state();
    
    // Saldırgan ve dürüst node'ların blockchain'lerini karşılaştır
    println!("\n--- SALDIRGAN VE DÜRÜST NODE KARŞILAŞTIRMASI ---");
    println!("Saldırgan Node ({})'un blockchain'i:", attacker_id);
    network.print_blockchain(attacker_id);
    
    println!("Dürüst Node ({})'un blockchain'i:", honest_node_id);
    network.print_blockchain(honest_node_id);
    
    println!("\nBlockchain Network Simulation Finished.");
}