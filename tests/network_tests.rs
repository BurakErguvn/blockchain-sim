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
        .create_transaction_with_fee(&recipient_address, 100_000_000, 0)
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
fn mempool_min_fee_rate_altindaki_islem_reddedilmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient_address = network.get_node_address(recipient_id);

    let tx = network.create_transaction_with_fee(sender_id, &recipient_address, 100_000_000, 0);
    assert!(tx.is_none());
    assert!(network.mempool.is_empty());
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

#[test]
fn rbf_lite_daha_yuksek_fee_ile_replace_etmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let sender_id =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let recipient_id = if sender_id == 0 { 1 } else { 0 };
    let recipient_address = network.get_node_address(recipient_id);

    let low_fee_tx = network
        .create_transaction_with_fee(sender_id, &recipient_address, 100_000_000, 1_000)
        .expect("dusuk fee tx mempoola girmeli");
    let high_fee_tx = network
        .create_transaction_with_fee(sender_id, &recipient_address, 100_000_000, 50_000)
        .expect("yuksek fee tx dusuk fee tx'i replace etmeli");

    assert_eq!(network.mempool.len(), 1);
    assert_eq!(network.mempool[0].id, high_fee_tx.id);
    assert_ne!(network.mempool[0].id, low_fee_tx.id);
}

#[test]
fn blok_seciminde_yuksek_fee_orani_once_gelmeli() {
    let mut network = setup_network(4);
    mine_genesis(&mut network);

    let sender_a =
        find_funded_node(&network).expect("Genesis sonrasi en az bir node fonlanmis olmali");
    let sender_b = if sender_a == 0 { 1 } else { 0 };
    let sender_b_address = network.get_node_address(sender_b);

    let _fund_tx = network
        .create_transaction_with_fee(sender_a, &sender_b_address, 200_000_000, 1_000)
        .expect("sender_b fonlanmali");

    for node in network.nodes.iter_mut() {
        node.is_validator = false;
    }
    network.nodes[0].is_validator = true;
    network.current_validator_id = Some(0);
    let _ = network.mine_block().expect("fonlama blogu uretilmeli");

    let recipient_id = if sender_b == 2 { 3 } else { 2 };
    let recipient_address = network.get_node_address(recipient_id);

    let low_fee_tx = network
        .create_transaction_with_fee(sender_a, &recipient_address, 100_000_000, 1_000)
        .expect("dusuk fee tx olusmali");
    let high_fee_tx = network
        .create_transaction_with_fee(sender_b, &recipient_address, 50_000_000, 50_000)
        .expect("yuksek fee tx olusmali");

    for node in network.nodes.iter_mut() {
        node.is_validator = false;
    }
    network.nodes[0].is_validator = true;
    network.current_validator_id = Some(0);
    let mined_block = network.mine_block().expect("blok uretilmeli");

    let included_ids: Vec<String> = mined_block
        .transactions
        .iter()
        .skip(1)
        .map(|tx| tx.id.clone())
        .collect();
    assert!(included_ids.iter().any(|id| id == &low_fee_tx.id));
    assert!(included_ids.iter().any(|id| id == &high_fee_tx.id));

    let high_pos = included_ids
        .iter()
        .position(|id| id == &high_fee_tx.id)
        .expect("yuksek fee tx blokta olmali");
    let low_pos = included_ids
        .iter()
        .position(|id| id == &low_fee_tx.id)
        .expect("dusuk fee tx blokta olmali");
    assert!(high_pos < low_pos);
}

#[test]
fn esit_uzunlukta_daha_yuksek_is_kaniti_olan_zincir_secilebilmeli() {
    let mut network = setup_network(3);
    mine_genesis(&mut network);

    let node_id = 0;
    let base_chain = network.nodes[node_id].blockchain.clone();
    let last_block = base_chain.last().expect("genesis olmali");
    let reward = network.nodes[node_id].mining_reward;
    let miner_address = network.nodes[node_id].wallet.get_address().to_string();

    let weak_coinbase = Transaction::new_coinbase(miner_address.clone(), reward);
    let mut weak_block = Block::new(
        last_block.index + 1,
        last_block.timestamp + 1,
        vec![weak_coinbase],
        last_block.hash.clone(),
    );
    weak_block.mine_block(network.difficulty);
    let mut weak_chain = base_chain.clone();
    weak_chain.push(weak_block.clone());

    let strong_coinbase = Transaction::new_coinbase(miner_address, reward);
    let mut strong_block = Block::new(
        last_block.index + 1,
        last_block.timestamp + 2,
        vec![strong_coinbase],
        last_block.hash.clone(),
    );
    strong_block.mine_block(network.difficulty + 1);
    let mut strong_chain = base_chain;
    strong_chain.push(strong_block.clone());

    network.nodes[node_id].update_blockchain(weak_chain, network.difficulty);
    let tip_after_weak = network.nodes[node_id]
        .blockchain
        .last()
        .expect("tip olmali")
        .hash
        .clone();
    assert_eq!(tip_after_weak, weak_block.hash);

    network.nodes[node_id].update_blockchain(strong_chain, network.difficulty);
    let tip_after_strong = network.nodes[node_id]
        .blockchain
        .last()
        .expect("tip olmali")
        .hash
        .clone();
    assert_eq!(tip_after_strong, strong_block.hash);
}

#[test]
fn fork_reorg_simulasyonu_tum_nodelari_kanonik_zincire_tasimali() {
    let mut network = setup_network(4);
    mine_genesis(&mut network);

    let reorg_depth = network
        .simulate_fork_and_reorg(0, 1)
        .expect("fork/reorg simulasyonu calismali");
    assert_eq!(reorg_depth, 1);

    let canonical_tip = network.nodes[0]
        .blockchain
        .last()
        .expect("tip olmali")
        .hash
        .clone();
    let canonical_len = network.nodes[0].blockchain.len();
    assert_eq!(canonical_len, 3);

    for node in &network.nodes {
        assert_eq!(node.blockchain.len(), canonical_len);
        assert_eq!(
            node.blockchain.last().expect("tip olmali").hash,
            canonical_tip
        );
    }
}
