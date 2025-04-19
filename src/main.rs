use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;

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
    for i in 0..network.node_count() {
        for j in (i + 1)..network.node_count() {
            network.connect_nodes(i, j);
        }
    }

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
    let manipulator_id = (network.current_val_id().unwrap() + 1 ) % network.node_count();
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

    //Validator hash'i değiştiriyor (bu kabul edilir)
    if let Some(validator_id) = network.current_val_id() {
        println!("Validator {} is trying to change the hash.", validator_id);
        network.try_manipulate_hash(validator_id, "888888".to_string());
        network.print_network_state();
    }
    
    // Bir node'un blockchain'ini manipüle etmeyi dene
    println!("\n--- NODE BLOCKCHAIN MANIPULATION TEST ---");
    let attacker_id = (network.current_val_id().unwrap() + 2) % network.node_count();
    println!("Node {} is trying to manipulate the blockchain.", attacker_id);
    network.try_manipulate_blockchain(attacker_id);
    
    // Manipüle edilen node'un blockchain'ini görüntüle
    network.print_blockchain(attacker_id);
    
    // Başka bir node'un (sağlam) blockchain'ini görüntüle
    let honest_node_id = (attacker_id + 1) % network.node_count();
    network.print_blockchain(honest_node_id);
    
    println!("Blockchain Network Simulation Finished.");
}