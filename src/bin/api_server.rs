use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use blockchain_sim::api;
use blockchain_sim::network::BlockchainNetwork;

fn bootstrap_network() -> BlockchainNetwork {
    let mut network = BlockchainNetwork::new();
    network.set_difficulty(2);
    network.set_block_time(60);

    for _ in 0..5 {
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
    let network = match BlockchainNetwork::load_from_disk(BlockchainNetwork::DEFAULT_STATE_PATH) {
        Ok(state) => {
            println!("Persisted network state yüklendi.");
            state
        }
        Err(err) => {
            println!("Persisted state yüklenemedi: {}. Yeni ağ başlatılıyor.", err);
            let network = bootstrap_network();
            if let Err(save_err) = network.save_to_disk(BlockchainNetwork::DEFAULT_STATE_PATH) {
                println!("Başlangıç state'i kaydedilemedi: {}", save_err);
            }
            network
        }
    };

    let app = api::router(Arc::new(Mutex::new(network)));
    let addr: SocketAddr = "0.0.0.0:3000"
        .parse()
        .expect("Sunucu adresi parse edilemedi");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("API listener açılamadı");

    println!("API server çalışıyor: http://{}", addr);
    axum::serve(listener, app)
        .await
        .expect("API server çalışırken hata oluştu");
}
