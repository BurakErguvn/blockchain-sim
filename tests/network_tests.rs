mod common;

use blockchain_sim::block::Block;
use blockchain_sim::transaction::Transaction;

use common::{find_funded_node, mine_genesis, setup_network};

#[test]
fn node_ekleme_ve_baglama_calismali() {
    let network = setup_network(3);

    assert_eq!(network.node_count(), 3);
    assert_eq!(network.nodes[0].connections.len(), 2);
    assert_eq!(network.nodes[1].connections.len(), 2);
    assert_eq!(network.nodes[2].connections.len(), 2);
}

#[test]
fn genesis_mining_sonrasi_tum_nodelarda_blok_olmali() {
    let mut network = setup_network(4);
    mine_genesis(&mut network);

    for node in &network.nodes {
        assert_eq!(node.blockchain.len(), 1);
    }
}

#[test]
fn gecersiz_sender_id_ile_transfer_olusmamali() {
    let mut network = setup_network(2);
    let recipient = network.get_node_address(1);

    let tx = network.create_transaction(99, &recipient, 1_000);
    assert!(tx.is_none());
}

#[test]
fn gecerli_transfer_ag_mempooluna_eklenmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient_address = network.get_node_address(recipient_id);

    let tx = network.create_transaction(sender_id, &recipient_address, 100_000_000);
    assert!(tx.is_some());
    assert_eq!(network.mempool.len(), 1);
}

#[test]
fn imzali_islem_manipule_edilirse_dogrulama_red_etmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient_address = network.get_node_address(recipient_id);

    let tx = network
        .create_transaction(sender_id, &recipient_address, 100_000_000)
        .expect("gecerli transaction olusmali");

    let mut tampered_tx = tx.clone();
    tampered_tx.outputs[0].amount += 1;
    tampered_tx.id = tampered_tx.calculate_hash();

    assert!(!network.nodes[sender_id].verify_transaction(&tampered_tx));
}

#[test]
fn mempool_double_spend_girisimi_reddedilmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient_address = network.get_node_address(recipient_id);

    let first_tx = network.create_transaction(sender_id, &recipient_address, 100_000_000);
    let second_tx = network.create_transaction(sender_id, &recipient_address, 50_000_000);

    assert!(first_tx.is_some());
    assert!(second_tx.is_none());
    assert_eq!(network.mempool.len(), 1);
}

#[test]
fn madenci_odulu_ucret_eklenince_artmali() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient_address = network.get_node_address(recipient_id);

    let mut fee_tx = network.nodes[sender_id]
        .wallet
        .create_transaction(&recipient_address, 100_000_000)
        .expect("islem olusmali");
    fee_tx.outputs[1].amount -= 1_000;
    fee_tx.id = fee_tx.calculate_hash();
    for i in 0..fee_tx.inputs.len() {
        let payload = fee_tx.signing_payload(i).expect("imza payload olusmali");
        fee_tx.inputs[i].signature = network.nodes[sender_id].wallet.sign(&payload);
    }
    assert!(network.nodes[sender_id].verify_transaction(&fee_tx));

    network.mempool.push(fee_tx);
    for node in network.nodes.iter_mut() {
        node.is_validator = false;
    }
    network.nodes[0].is_validator = true;
    network.current_validator_id = Some(0);

    let mined_block = network.mine_block().expect("blok uretilmeli");
    let coinbase_reward = mined_block.transactions[0].get_total_output_amount();
    assert_eq!(coinbase_reward, network.nodes[0].mining_reward + 1_000);
}

#[test]
fn asiri_coinbase_odulu_olan_blok_reddedilmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let node = &network.nodes[0];
    let last_block = node.blockchain.last().expect("genesis olmali");
    let fake_coinbase = Transaction::new_coinbase(
        node.wallet.get_address().to_string(),
        node.mining_reward + 1,
    );

    let mut fake_block = Block::new(
        last_block.index + 1,
        last_block.timestamp + 1,
        vec![fake_coinbase],
        last_block.hash.clone(),
    );
    fake_block.mine_block(network.difficulty);

    assert!(!node.is_valid_new_block(&fake_block, network.difficulty));
}

#[test]
fn chain_sync_sirasinda_wallet_kimligi_ve_bakiye_korunmali() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let target_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let original_address = network.nodes[target_id].wallet.get_address().to_string();
    let original_balance = network.nodes[target_id].wallet.get_balance();

    let base_chain = network.nodes[target_id].blockchain.clone();
    let last_block = base_chain.last().expect("zincirde genesis olmali");
    let reward = network.nodes[target_id].mining_reward;
    let coinbase = Transaction::new_coinbase("external-miner".to_string(), reward);

    let mut new_block = Block::new(
        last_block.index + 1,
        last_block.timestamp + 1,
        vec![coinbase],
        last_block.hash.clone(),
    );
    new_block.mine_block(network.difficulty);

    let mut longer_chain = base_chain;
    longer_chain.push(new_block);

    network.nodes[target_id].update_blockchain(longer_chain, network.difficulty);
    let synced_node = &network.nodes[target_id];

    assert_eq!(synced_node.wallet.get_address(), original_address);
    assert_eq!(synced_node.wallet.get_balance(), original_balance);

    let expected_balance: u64 = synced_node
        .utxo_set
        .values()
        .filter(|utxo| utxo.recipient_address == original_address)
        .map(|utxo| utxo.amount)
        .sum();
    assert_eq!(synced_node.wallet.get_balance(), expected_balance);
}

#[test]
fn chain_sync_sonrasi_mempooldaki_harcanmis_tx_temizlenmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient_address = network.get_node_address(recipient_id);

    let pending_tx = network.nodes[sender_id]
        .create_transaction(&recipient_address, 100_000_000)
        .expect("gonderici mempoola tx ekleyebilmeli");
    assert_eq!(network.nodes[sender_id].mempool.len(), 1);

    let base_chain = network.nodes[sender_id].blockchain.clone();
    let last_block = base_chain.last().expect("zincirde genesis olmali");
    let reward = network.nodes[sender_id].mining_reward;
    let coinbase = Transaction::new_coinbase("sync-miner".to_string(), reward);

    let mut sync_block = Block::new(
        last_block.index + 1,
        last_block.timestamp + 1,
        vec![coinbase, pending_tx],
        last_block.hash.clone(),
    );
    sync_block.mine_block(network.difficulty);

    let mut longer_chain = base_chain;
    longer_chain.push(sync_block);
    network.nodes[sender_id].update_blockchain(longer_chain, network.difficulty);

    assert!(network.nodes[sender_id].mempool.is_empty());
}

#[test]
fn broadcast_blockchain_sonrasi_ag_mempoolu_da_temizlenmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient_address = network.get_node_address(recipient_id);

    let pending_tx = network
        .create_transaction(sender_id, &recipient_address, 100_000_000)
        .expect("ag mempooluna tx eklenmeli");
    assert_eq!(network.mempool.len(), 1);

    let base_chain = network.nodes[sender_id].blockchain.clone();
    let last_block = base_chain.last().expect("zincirde genesis olmali");
    let reward = network.nodes[sender_id].mining_reward;
    let coinbase = Transaction::new_coinbase("sync-miner".to_string(), reward);

    let mut sync_block = Block::new(
        last_block.index + 1,
        last_block.timestamp + 1,
        vec![coinbase, pending_tx],
        last_block.hash.clone(),
    );
    sync_block.mine_block(network.difficulty);

    let mut longer_chain = base_chain;
    longer_chain.push(sync_block);
    network.broadcast_blockchain(longer_chain);

    assert!(network.mempool.is_empty());
}
