mod common;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use blockchain_sim::network::BlockchainNetwork;

use common::{mine_genesis, setup_network};

fn temp_state_path(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("sistem zamani alinabilmeli")
        .as_nanos();
    std::env::temp_dir().join(format!("{}_{}.json", prefix, now))
}

#[test]
fn persistence_roundtrip_wallet_ve_zinciri_korumali() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);
    let _ = network
        .mine_block()
        .expect("en az bir blok daha uretilmeli");

    let pre_load_addresses: Vec<String> = network
        .nodes
        .iter()
        .map(|node| node.wallet.get_address().to_string())
        .collect();
    let pre_load_balances: Vec<u64> = network
        .nodes
        .iter()
        .map(|node| node.wallet.get_balance())
        .collect();
    let pre_load_hashes: Vec<Vec<String>> = (0..network.node_count())
        .map(|id| network.get_node_blockchain_hashes(id))
        .collect();

    let state_path = temp_state_path("blockchain_state_roundtrip");
    network
        .save_to_disk(state_path.to_str().expect("path utf8 olmali"))
        .expect("state diske yazilabilmeli");

    let loaded_network =
        BlockchainNetwork::load_from_disk(state_path.to_str().expect("path utf8 olmali"))
            .expect("state diskten yuklenebilmeli");

    assert_eq!(loaded_network.node_count(), network.node_count());
    assert_eq!(loaded_network.difficulty, network.difficulty);
    assert_eq!(loaded_network.block_time, network.block_time);
    assert_eq!(loaded_network.mempool.len(), 0);

    for i in 0..loaded_network.node_count() {
        assert_eq!(
            loaded_network.nodes[i].wallet.get_address(),
            pre_load_addresses[i]
        );
        assert_eq!(
            loaded_network.nodes[i].wallet.get_balance(),
            pre_load_balances[i]
        );
        assert_eq!(
            loaded_network.get_node_blockchain_hashes(i),
            pre_load_hashes[i]
        );
    }

    let _ = fs::remove_file(state_path);
}

#[test]
fn bozuk_state_dosyasi_recoverable_hata_dondurmeli() {
    let state_path = temp_state_path("blockchain_state_broken");
    fs::write(&state_path, "{ this is not valid json ").expect("test dosyasi yazilabilmeli");

    let loaded = BlockchainNetwork::load_from_disk(state_path.to_str().expect("path utf8 olmali"));
    assert!(loaded.is_err());

    let _ = fs::remove_file(state_path);
}
