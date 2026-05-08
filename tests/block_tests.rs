use blockchain_sim::block::Block;
use blockchain_sim::transaction::Transaction;

#[test]
fn block_hash_nonce_degistiginde_degismeli() {
    let tx = Transaction::new_coinbase("alice".to_string(), 50);
    let mut block = Block::new(0, 1_717_171_717, vec![tx], "0".to_string());

    let initial_hash = block.hash.clone();
    block.nonce += 1;
    let recalculated_hash = block.calculate_hash();

    assert_ne!(initial_hash, recalculated_hash);
}

#[test]
fn mine_block_hash_zorluk_saglamali() {
    let tx = Transaction::new_coinbase("miner".to_string(), 50);
    let mut block = Block::new(1, 1_717_171_718, vec![tx], "prev-hash".to_string());

    block.mine_block(2);

    assert!(block.hash.starts_with("00"));
}

#[test]
fn merkle_root_islem_oldugunda_bos_olmamali() {
    let tx1 = Transaction::new_coinbase("user-a".to_string(), 50);
    let tx2 = Transaction::new_coinbase("user-b".to_string(), 20);
    let block = Block::new(2, 1_717_171_719, vec![tx1, tx2], "prev-hash".to_string());

    assert!(!block.merkle_root.is_empty());
    assert_ne!(block.merkle_root, "0");
}
