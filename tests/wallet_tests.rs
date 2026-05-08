use blockchain_sim::transaction::UTXO;
use blockchain_sim::wallet::Wallet;

#[test]
fn bakiye_yetersizse_islem_olusmamali() {
    let wallet = Wallet::new();
    let tx = wallet.create_transaction("receiver-address", 1_000);

    assert!(tx.is_none());
}

#[test]
fn yeterli_bakiyede_para_ustu_cikisi_olusmali() {
    let mut wallet = Wallet::new();
    let owner_address = wallet.get_address().to_string();

    wallet.add_utxo(UTXO {
        transaction_id: "genesis-tx".to_string(),
        output_index: 0,
        amount: 5_000,
        recipient_address: owner_address,
    });

    let tx = wallet
        .create_transaction("receiver-address", 3_000)
        .expect("Islem olusmali");

    assert_eq!(tx.inputs.len(), 1);
    assert_eq!(tx.outputs.len(), 2);
    assert_eq!(tx.outputs[0].amount, 3_000);
    assert_eq!(tx.outputs[1].amount, 2_000);
}
