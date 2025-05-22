use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

mod node;
mod network;
mod block;
mod wallet;
mod transaction;

use network::BlockchainNetwork;
use block::Block;

// Blok oluşturulduğunda gönderilecek mesaj için kanal
struct BlockchainMessage {
    block: Block,
    validator_id: usize,
    next_validator_id: usize,
}

fn main() {
    // Blockchain ağını oluştur
    let network = Arc::new(Mutex::new(BlockchainNetwork::new()));
    
    // Blok mesajları için kanal oluştur
    let (block_sender, block_receiver) = mpsc::channel::<BlockchainMessage>();
    
    // Yeni bir terminal aç ve blok bilgilerini göster
    let block_display_thread = thread::spawn(move || {
        println!("Blok görüntüleme terminali başlatılıyor...");
        
        // Yeni bir dosya oluştur ve blok bilgilerini oraya yaz
        let log_file_path = "/tmp/blockchain_blocks.log";
        let _ = std::fs::write(log_file_path, "Blockchain Blok Görüntüleyici - Yeni bloklar burada görüntülenecek...\n");
        
        // Yeni bir terminal aç ve log dosyasını göster
        #[cfg(target_os = "linux")]
        let _ = Command::new("x-terminal-emulator")
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(format!("tail -f {} || read", log_file_path))
            .spawn();
        
        // Blok mesajlarını al ve dosyaya yaz
        while let Ok(message) = block_receiver.recv() {
            // Ana terminalde kısa bir bilgi göster
            println!("Yeni blok oluşturuldu: {} (Node {})", message.block.hash, message.validator_id);
            
            // Detaylı bilgileri log dosyasına yaz
            let log_entry = format!(
                "\n=== YENİ BLOK OLUŞTURULDU ===\n\
                Blok Hash: {}\n\
                Blok İndeksi: {}\n\
                Blok Zaman Damgası: {}\n\
                Blok İşlem Sayısı: {}\n\
                Oluşturan Madenci: Node {}\n\
                Yeni Madenci: Node {}\n\
                ==============================\n",
                message.block.hash,
                message.block.index,
                message.block.timestamp,
                message.block.transactions.len(),
                message.validator_id,
                message.next_validator_id
            );
            
            // Log dosyasına ekle
            if let Err(e) = std::fs::OpenOptions::new()
                .append(true)
                .open(log_file_path)
                .and_then(|mut file| std::io::Write::write_all(&mut file, log_entry.as_bytes()))
            {
                println!("Log dosyasına yazılamadı: {}", e);
            }
            
            // Masaüstü bildirimi gönder
            #[cfg(target_os = "linux")]
            let _ = Command::new("notify-send")
                .arg("Yeni Blok Oluşturuldu")
                .arg(format!("Hash: {}\nValidator: Node {}", message.block.hash, message.validator_id))
                .spawn();
        }
    });
    
    // Blockchain ağını başlat
    {
        let mut network_lock = network.lock().unwrap();
        
        // Madencilik zorluğunu ayarla (2 = hash'in başında 2 tane 0 olmalı)
        network_lock.set_difficulty(2);
        
        // Block time'ı ayarla (gerçekçi bir simülasyon için)
        network_lock.set_block_time(60); // 60 saniye
        
        println!("Blockchain simülasyonu başlatılıyor...");

        // 5 tane node oluştur
        println!("\n--- NODE'LAR OLUŞTURULUYOR ---");
        for _i in 0..5 {
            let node_id = network_lock.add_node();
            println!("Node {} oluşturuldu", node_id);
        }

        // Node'ları birbirine bağla (tam bağlı ağ)
        println!("\n--- NODE'LAR ARASI BAĞLANTILAR KURULUYOR ---");
        for i in 0..network_lock.node_count() {
            for j in (i + 1)..network_lock.node_count() {
                network_lock.connect_nodes(i, j);
            }
        }
        println!("Tüm node'lar arasında bağlantılar kuruldu.");

        // Başlangıç durumunu görüntüle
        println!("\n--- BLOCKCHAIN AĞI OLUŞTURULDU ---");
        network_lock.print_network_state();

        // İlk madenci seç
        println!("\n--- MADENCİ SEÇİLİYOR ---");
        network_lock.select_random_validator();
        let validator_id = network_lock.current_val_id().unwrap();
        println!("Node {} madenci olarak seçildi.", validator_id);
        
        // Otomatik madencilik işlemini başlat
        println!("\n--- OTOMATİK MADENCİLİK BAŞLATILIYOR ---");
        match network_lock.start_automatic_mining() {
            Ok(_) => println!("Otomatik madencilik başlatıldı. Block time: {} saniye", network_lock.block_time),
            Err(e) => {
                println!("Madencilik başlatılamadı: {}", e);
                return;
            }
        };
        
        // Genesis bloğunu oluştur
        println!("Genesis bloğu oluşturuluyor...");
        if let Some(block) = network_lock.mine_block() {
            println!("Genesis bloğu oluşturuldu: {}", block.hash);
            
            // Blok mesajını gönder
            let message = BlockchainMessage {
                block: block.clone(),
                validator_id: validator_id, // Bloğu oluşturan madenci (mevcut validator)
                next_validator_id: network_lock.current_val_id().unwrap(), // Yeni seçilen madenci
            };
            let _ = block_sender.send(message);
        } else {
            println!("Genesis bloğu oluşturulamadı!");
            return;
        }
        
        // Genesis blok sonrası ağın durumunu görüntüle
        println!("\n--- GENESIS BLOĞU SONRASI AĞ DURUMU ---");
        network_lock.print_network_state();
    }
    
    // Blockchain ağı için bir klon oluştur
    let network_clone = Arc::clone(&network);
    let block_sender_clone = block_sender.clone();
    
    // Madencilik thread'i
    let mining_thread = thread::spawn(move || {
        // Son blok oluşturma zamanını takip et
        let mut last_block_time = SystemTime::now();
        
        loop {
            // Kısa aralıklarla kontrol et
            thread::sleep(Duration::from_secs(1));
            
            let mut network_lock = network_clone.lock().unwrap();
            let block_time_secs = network_lock.block_time;
            
            // Şu anki zamanı al
            let now = SystemTime::now();
            
            // Son blok oluşturulduğundan beri geçen süreyi hesapla
            let elapsed = now.duration_since(last_block_time).unwrap_or(Duration::from_secs(0));
            
            // Block time geçtiyse yeni blok oluştur
            if elapsed.as_secs() >= block_time_secs {
                // Mine_block öncesinde şu anki validator'u kaydet
                let current_validator = network_lock.current_val_id().unwrap_or(0);
                
                // Yeni bir blok oluşturulduğunda, blok mesajını gönder
                if let Some(block) = network_lock.mine_block() {
                    // Yeni validator ID'sini al
                    let new_validator = network_lock.current_val_id().unwrap_or(0);
                    
                    // Blok mesajını gönder
                    let message = BlockchainMessage {
                        block: block.clone(),
                        validator_id: current_validator, // Bloğu oluşturan madenci (eski validator)
                        next_validator_id: new_validator, // Yeni seçilen madenci
                    };
                    let _ = block_sender_clone.send(message);
                    
                    // Son blok zamanını güncelle
                    last_block_time = now;
                }
            }
        }
    });
    
    // Komut arayüzü
    println!("\n=== BLOCKCHAIN KOMUT ARAYÜZÜ ===");
    println!("Kullanabileceğiniz komutlar:");
    println!("1. bakiye <node_id> - Belirtilen node'un bakiyesini gösterir");
    println!("2. transfer <gönderen_id> <alıcı_id> <miktar> - Coin transferi yapar");
    println!("3. durum - Ağın genel durumunu gösterir");
    println!("4. blockchain <node_id> - Belirtilen node'un blockchain'ini gösterir");
    println!("5. mempool - Mempool'daki işlemleri gösterir");
    println!("6. çıkış - Simülasyonu sonlandır");
    println!("==============================\n");
    
    // Komut döngüsü
    loop {
        print!("Komut> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "bakiye" => {
                if parts.len() < 2 {
                    println!("Hata: Node ID belirtilmedi. Kullanım: bakiye <node_id>");
                    continue;
                }
                
                if let Ok(node_id) = parts[1].parse::<usize>() {
                    let network_lock = network.lock().unwrap();
                    
                    if node_id >= network_lock.node_count() {
                        println!("Hata: Geçersiz Node ID. 0-{} arasında bir değer girin.", network_lock.node_count() - 1);
                        continue;
                    }
                    
                    let balance = network_lock.nodes[node_id].wallet.get_balance();
                    println!("Node {} bakiyesi: {} coin", node_id, balance as f64 / 100_000_000.0);
                } else {
                    println!("Hata: Geçersiz Node ID formatı. Sayısal bir değer girin.");
                }
            },
            "transfer" => {
                if parts.len() < 4 {
                    println!("Hata: Eksik parametreler. Kullanım: transfer <gönderen_id> <alıcı_id> <miktar>");
                    continue;
                }
                
                if let (Ok(sender_id), Ok(receiver_id), Ok(amount)) = (
                    parts[1].parse::<usize>(),
                    parts[2].parse::<usize>(),
                    parts[3].parse::<f64>()
                ) {
                    let mut network_lock = network.lock().unwrap();
                    
                    if sender_id >= network_lock.node_count() || receiver_id >= network_lock.node_count() {
                        println!("Hata: Geçersiz Node ID. 0-{} arasında bir değer girin.", network_lock.node_count() - 1);
                        continue;
                    }
                    
                    // Coin miktarını satoshi birimine çevir (1 coin = 100,000,000 satoshi)
                    let amount_satoshi = (amount * 100_000_000.0) as u64;
                    
                    // Alıcı adresini önceden al
                    let receiver_address = network_lock.get_node_address(receiver_id).clone();
                    
                    println!("Node {} -> Node {}: {} coin transfer ediliyor", 
                        sender_id, receiver_id, amount);
                    
                    let tx = network_lock.create_transaction(
                        sender_id, 
                        &receiver_address, 
                        amount_satoshi
                    );
                    
                    if let Some(_) = tx {
                        println!("İşlem oluşturuldu ve mempool'a eklendi");
                    } else {
                        println!("İşlem oluşturulamadı! Bakiye yetersiz olabilir.");
                    }
                } else {
                    println!("Hata: Geçersiz parametre formatı. Sayısal değerler girin.");
                }
            },
            "durum" => {
                let network_lock = network.lock().unwrap();
                network_lock.print_network_state();
            },
            "blockchain" => {
                if parts.len() < 2 {
                    println!("Hata: Node ID belirtilmedi. Kullanım: blockchain <node_id>");
                    continue;
                }
                
                if let Ok(node_id) = parts[1].parse::<usize>() {
                    let network_lock = network.lock().unwrap();
                    
                    if node_id >= network_lock.node_count() {
                        println!("Hata: Geçersiz Node ID. 0-{} arasında bir değer girin.", network_lock.node_count() - 1);
                        continue;
                    }
                    
                    network_lock.print_blockchain(node_id);
                } else {
                    println!("Hata: Geçersiz Node ID formatı. Sayısal bir değer girin.");
                }
            },
            "mempool" => {
                let network_lock = network.lock().unwrap();
                
                println!("\n--- MEMPOOL'DAKİ İŞLEMLER ---");
                if network_lock.mempool.is_empty() {
                    println!("Mempool boş.");
                } else {
                    for (i, tx) in network_lock.mempool.iter().enumerate() {
                        println!("\nİşlem {}: ID: {}", i+1, tx.id);
                        println!("Gönderen: {}", tx.inputs[0].sender_address);
                        println!("Alıcı: {}", tx.outputs[0].recipient_address);
                        println!("Miktar: {} coin", tx.outputs[0].amount as f64 / 100_000_000.0);
                    }
                }
                println!("-----------------------------\n");
            },
            "çıkış" | "exit" | "quit" => {
                println!("Simülasyon sonlandırılıyor...");
                
                // Madenciliği durdur
                {
                    let mut network_lock = network.lock().unwrap();
                    match network_lock.stop_automatic_mining() {
                        Ok(_) => println!("Madencilik durduruldu"),
                        Err(e) => println!("Madencilik durdurulamadı: {}", e),
                    }
                }
                
                // Thread'leri sonlandır
                drop(block_sender); // Kanalı kapat
                
                // Programı sonlandır
                std::process::exit(0);
            },
            _ => {
                println!("Bilinmeyen komut: {}", parts[0]);
                println!("Kullanabileceğiniz komutlar: bakiye, transfer, durum, blockchain, mempool, çıkış");
            }
        }
    }
    
    // Thread'leri sonlandır
    drop(block_sender);
    let _ = mining_thread.join();
    let _ = block_display_thread.join();
    
    println!("Blockchain Simülasyonu sonlandırıldı.");
}