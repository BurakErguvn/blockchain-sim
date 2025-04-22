use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;
use rand;
use rand::Rng;
use rand::thread_rng;
use sha2::{Sha256, Digest};

mod node;

use node::BlockchainNetwork;

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

    //İlk işlemi oluştur
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let transaction = format!("Tx {}-{}","Initial_transaction",timestamp );
    network.create_transaction(&transaction);
    
    // İlk node'un blockchain'ini görüntüle
    if network.node_count() > 0 {
        network.print_blockchain(0);
    }
    
    network.print_network_state();

    //Biraz bekleyelim
    thread::sleep(Duration::from_secs(2));

    //Bir node'un hash'ini değiştirmeye çalış (validator olmayan)
    let manipulator_id = if let Some(validator_id) = network.current_val_id() {
        (validator_id + 1) % network.node_count()
    } else {
        // Validator yoksa, rasgele bir node seç
        network.select_random_validator(); // Yeni bir validator seç
        println!("No validator found. Selecting a new one.");
        // Ve manipülatör olarak bir sonraki node'u seç
        if let Some(validator_id) = network.current_val_id() {
            (validator_id + 1) % network.node_count()
        } else {
            println!("Failed to select a validator. Using node 0 as manipulator.");
            0 // Node 0'ı manipülatör olarak kullan
        }
    };
    
    println!("Node {} is trying to change the hash.", manipulator_id);
    network.try_manipulate_hash(manipulator_id, "999999".to_string());
    network.print_network_state();
    
    //Rasgele bir validator seç
    network.select_random_validator();
    
    //Yeni bir işlem oluştur
    let timestamp2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let transaction2 = format!("Tx {}-{}","Second_transaction",timestamp2 );
    network.create_transaction(&transaction2);
    
    // İlk node'un blockchain'ini tekrar görüntüle
    if network.node_count() > 0 {
        network.print_blockchain(0);
    }
    
    network.print_network_state();
    
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
    
    network.print_blockchain(attacker_id);
    
    network.print_blockchain(honest_node_id);
    
    println!("\nBlockchain Network Simulation Finished.");
}