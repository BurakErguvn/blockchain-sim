mod common;

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
