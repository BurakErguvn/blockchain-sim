use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;
use rand;
use rand::Rng;

mod node;
mod network;
mod block;
mod wallet;
mod transaction;

use network::BlockchainNetwork;

fn main() {
    let mut network = BlockchainNetwork::new();

    // Madencilik zorluğunu ayarla (2 = hash'in başında 2 tane 0 olmalı)
    network.set_difficulty(2);

    // 5 tane node oluştur
    println!("\n--- NODE'LAR OLUŞTURULUYOR ---");
    for i in 0..5 {
        let node_id = network.add_node();
        println!("Node {} oluşturuldu", node_id);
    }

    // Node'ları birbirine bağla (tam bağlı ağ)
    println!("\n--- NODE'LAR ARASI BAĞLANTILAR KURULUYOR ---");
    for i in 0..network.node_count() {
        for j in (i + 1)..network.node_count() {
            network.connect_nodes(i, j);
        }
    }
    println!("Tüm node'lar arasında bağlantılar kuruldu.");

    // Başlangıç durumunu görüntüle
    println!("\n--- BLOCKCHAIN AĞI OLUŞTURULDU ---");
    network.print_network_state();

    // İlk madenci seç ve genesis bloğu kaz
    println!("\n--- GENESIS BLOĞU KAZILIYOR ---");
    network.select_random_validator();
    let validator_id = network.current_val_id().unwrap();
    println!("Node {} madenci olarak seçildi ve genesis bloğunu kazacak.", validator_id);
    
    // Genesis bloğunu kaz
    if let Some(block) = network.mine_block() {
        println!("Genesis blok oluşturuldu");
    } else {
        println!("Genesis blok oluşturulamadı!");
        return; // Genesis blok oluşturulamazsa programdan çık
    }
    
    // Genesis blok sonrası ağın durumunu görüntüle
    println!("\n--- GENESIS BLOĞU SONRASI AĞ DURUMU ---");
    network.print_network_state();
    
    // Şimdi işlemler oluştur
    println!("\n--- İŞLEMLER OLUŞTURULUYOR ---");
    
    // İlk işlem: Madenci node'dan Node 1'e 5 coin transfer
    let sender_id = validator_id; // Madenci node'un ID'si
    let receiver_id = (validator_id + 1) % network.node_count(); // Bir sonraki node
    let amount = 5_0000_0000; // 5 coin
    
    println!("\nNode {} -> Node {}: {} coin transfer ediliyor", 
        sender_id, receiver_id, amount as f64 / 100_000_000.0);
    
    let tx1 = network.create_transaction(
        sender_id, 
        &network.get_node_address(receiver_id), 
        amount
    );
    
    if let Some(_) = tx1 {
        println!("İşlem oluşturuldu");
    } else {
        println!("İşlem oluşturulamadı!");
    }
    
    // Yeni bir madenci seç
    println!("\n--- YENİ MADENCİ SEÇİLİYOR ---");
    network.select_random_validator();
    let new_validator_id = network.current_val_id().unwrap();
    println!("Node {} madenci olarak seçildi.", new_validator_id);
    
    // İkinci blok için madencilik yap
    println!("\n--- İKİNCİ BLOK İÇİN MADENCİLİK YAPILIYOR ---");
    if let Some(block) = network.mine_block() {
        println!("Yeni blok oluşturuldu: {}", block.hash);
        println!("Blok içindeki işlem sayısı: {}", block.transactions.len());
        
        // İşlemleri göster
        for (i, tx) in block.transactions.iter().enumerate() {
            println!("İşlem {}: {}", i, tx.id);
            if i == 0 {
                println!("  (Coinbase işlemi - Madencilik ödülü)");
            }
        }
    } else {
        println!("Blok oluşturulamadı!");
    }
    
    // İkinci blok sonrası ağın durumunu görüntüle
    println!("\n--- İKİNCİ BLOK SONRASI AĞ DURUMU ---");
    network.print_network_state();
    
    // Şimdi Node 1'in parası var, bir işlem deneyelim
    println!("\n--- ALICI NODE'DAN İŞLEM DENEMESİ ---");
    let sender_id = receiver_id; // Önceki işlemde para alan node
    let new_receiver_id = (receiver_id + 1) % network.node_count(); // Bir sonraki node
    let amount = 2_0000_0000; // 2 coin
    
    println!("Node {} -> Node {}: {} coin transfer ediliyor", 
        sender_id, new_receiver_id, amount as f64 / 100_000_000.0);
    
    let tx3 = network.create_transaction(
        sender_id, 
        &network.get_node_address(new_receiver_id), 
        amount
    );
    
    if let Some(_) = tx3 {
        println!("İşlem oluşturuldu");
    } else {
        println!("İşlem oluşturulamadı!");
    }
    
    // Yeni bir madenci seç
    println!("\n--- YENİ MADENCİ SEÇİLİYOR ---");
    network.select_random_validator();
    let third_validator_id = network.current_val_id().unwrap();
    println!("Node {} madenci olarak seçildi.", third_validator_id);
    
    // Üçüncü blok için madencilik yap
    println!("\n--- ÜÇÜNCÜ BLOK İÇİN MADENCİLİK YAPILIYOR ---");
    if let Some(block) = network.mine_block() {
        println!("Yeni blok oluşturuldu: {}", block.hash);
        println!("Blok içindeki işlem sayısı: {}", block.transactions.len());
        
        // İşlemleri göster
        for (i, tx) in block.transactions.iter().enumerate() {
            println!("İşlem {}: {}", i, tx.id);
            if i == 0 {
                println!("  (Coinbase işlemi - Madencilik ödülü)");
            }
        }
    } else {
        println!("Blok oluşturulamadı!");
    }
    
    // Son durumu görüntüle
    println!("\n--- SON AĞ DURUMU ---");
    network.print_network_state();
    
    // Tüm node'ların blockchain'lerini görüntüle
    println!("\n--- TÜM NODE'LARIN BLOCKCHAIN'LERİ ---");
    for i in 0..network.node_count() {
        network.print_blockchain(i);
    }
    
    println!("\nBlockchain Network Simulation Finished.");
}