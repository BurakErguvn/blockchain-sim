use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use blockchain_sim::api;
use blockchain_sim::config::Settings;
use blockchain_sim::network::BlockchainNetwork;

fn bootstrap_network(settings: &Settings) -> BlockchainNetwork {
    let mut network = BlockchainNetwork::new();
    network.set_difficulty(settings.network.difficulty);
    network.set_block_time(settings.network.block_time_seconds);

    for _ in 0..settings.app.initial_node_count {
        network.add_node();
    }

    for i in 0..network.node_count() {
        for j in (i + 1)..network.node_count() {
            network.connect_nodes(i, j);
        }
    }

    network.select_random_validator();
    let _ = network.mine_block();
    network
}

#[tokio::main]
async fn main() {
    let settings = match Settings::load_default() {
        Ok(settings) => settings,
        Err(err) => {
            println!(
                "Config yüklenemedi: {}. Varsayılan ayarlarla devam ediliyor.",
                err
            );
            Settings::default()
        }
    };

    let state_path = settings.persistence.state_path.clone();
    let network = match BlockchainNetwork::load_from_disk(&state_path) {
        Ok(state) => {
            println!("Persisted network state yüklendi.");
            state
        }
        Err(err) => {
            println!(
                "Persisted state yüklenemedi: {}. Yeni ağ başlatılıyor.",
                err
            );
            let network = bootstrap_network(&settings);
            if let Err(save_err) = network.save_to_disk(&state_path) {
                println!("Başlangıç state'i kaydedilemedi: {}", save_err);
            }
            network
        }
    };

    let app = api::router(Arc::new(Mutex::new(network)), state_path);
    let addr_str = format!("{}:{}", settings.api.bind_host, settings.api.bind_port);
    let addr: SocketAddr = addr_str.parse().expect("Sunucu adresi parse edilemedi");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("API listener açılamadı");

    println!("API server çalışıyor: http://{}", addr);
    axum::serve(listener, app)
        .await
        .expect("API server çalışırken hata oluştu");
}
