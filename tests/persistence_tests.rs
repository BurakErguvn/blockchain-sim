mod common;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use blockchain_sim::block::Block;
use blockchain_sim::network::BlockchainNetwork;
use serde::Serialize;

use common::{find_funded_node, mine_genesis, setup_network};

#[derive(Serialize)]
struct PersistedNodeStateV1Fixture {
    id: usize,
    connections: Vec<usize>,
    mining_reward: u64,
    blockchain: Vec<Block>,
    wallet_private_key_hex: String,
}

#[derive(Serialize)]
struct PersistedNetworkStateV1Fixture {
    schema_version: u32,
    difficulty: usize,
    block_time: u64,
    last_block_time: u64,
    current_validator_id: Option<usize>,
    max_mempool_bytes: usize,
    min_fee_rate_sat_per_kb: u64,
    replacement_increment_sat_per_kb: u64,
    nodes: Vec<PersistedNodeStateV1Fixture>,
}

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

#[test]
fn mempool_state_restart_sonrasi_korunmali_ve_revalidate_edilmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id = find_funded_node(&network).expect("fonlu node olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient = network.get_node_address(recipient_id);
    let tx = network
        .create_transaction(sender_id, &recipient, 100_000_000)
        .expect("gecerli tx olusmali");
    assert_eq!(network.mempool.len(), 1);

    let state_path = temp_state_path("blockchain_state_mempool");
    network
        .save_to_disk(state_path.to_str().expect("path utf8 olmali"))
        .expect("state diske yazilabilmeli");
    let loaded = BlockchainNetwork::load_from_disk(state_path.to_str().expect("path utf8 olmali"))
        .expect("state yuklenebilmeli");

    assert_eq!(loaded.mempool.len(), 1);
    assert_eq!(loaded.mempool[0].id, tx.id);
    for node in &loaded.nodes {
        assert!(node.mempool.iter().any(|entry| entry.id == tx.id));
    }

    let _ = fs::remove_file(state_path);
}

#[test]
fn v1_schema_state_yuklenip_migrate_edilebilmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);
    let _ = network.mine_block().expect("ek blok uretilmeli");

    let legacy_nodes: Vec<PersistedNodeStateV1Fixture> = network
        .nodes
        .iter()
        .map(|node| PersistedNodeStateV1Fixture {
            id: node.id,
            connections: node.connections.clone(),
            mining_reward: node.mining_reward,
            blockchain: node.blockchain.clone(),
            wallet_private_key_hex: node.wallet.private_key_hex(),
        })
        .collect();

    let legacy_state = PersistedNetworkStateV1Fixture {
        schema_version: 1,
        difficulty: network.difficulty,
        block_time: network.block_time,
        last_block_time: network.last_block_time,
        current_validator_id: network.current_validator_id,
        max_mempool_bytes: network.max_mempool_bytes,
        min_fee_rate_sat_per_kb: network.min_fee_rate_sat_per_kb,
        replacement_increment_sat_per_kb: network.replacement_increment_sat_per_kb,
        nodes: legacy_nodes,
    };

    let state_path = temp_state_path("blockchain_state_v1_legacy");
    let legacy_json = serde_json::to_vec_pretty(&legacy_state).expect("legacy json olusmali");
    fs::write(&state_path, legacy_json).expect("legacy dosya yazilabilmeli");

    let loaded = BlockchainNetwork::load_from_disk(state_path.to_str().expect("path utf8 olmali"))
        .expect("v1 state migrate edilerek yuklenebilmeli");
    assert_eq!(loaded.node_count(), network.node_count());
    assert_eq!(loaded.mempool.len(), 0);
    for i in 0..loaded.node_count() {
        assert_eq!(
            loaded.nodes[i].wallet.get_address(),
            network.nodes[i].wallet.get_address()
        );
        assert_eq!(
            loaded.get_node_blockchain_hashes(i),
            network.get_node_blockchain_hashes(i)
        );
    }

    let _ = fs::remove_file(state_path);
}
