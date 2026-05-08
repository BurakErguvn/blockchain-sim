use blockchain_sim::transaction::{get_utxo_id, Transaction, TxInput, TxOutput, UTXO};

#[test]
fn coinbase_transaction_gecerli_olmali() {
    let tx = Transaction::new_coinbase("receiver-address".to_string(), 5_000);
    let utxo_set: Vec<UTXO> = Vec::new();

    assert!(tx.is_valid(&utxo_set));
}

#[test]
fn output_inputtan_buyukse_transaction_gecersiz_olmali() {
    let source_utxo = UTXO {
        transaction_id: "source-tx".to_string(),
        output_index: 0,
        amount: 1_000,
        recipient_address: "sender-address".to_string(),
    };
    let input = TxInput {
        utxo_id: get_utxo_id(&source_utxo.transaction_id, source_utxo.output_index),
        utxo_output_index: source_utxo.output_index,
        signature: Vec::new(),
        sender_address: "sender-address".to_string(),
    };
    let outputs = vec![TxOutput {
        amount: 1_500,
        recipient_address: "receiver-address".to_string(),
    }];

    let tx = Transaction::new(vec![input], outputs);
    assert!(!tx.is_valid(&[source_utxo]));
}

#[test]
fn input_ve_output_toplamlari_hesaplanmali() {
    let source_utxo = UTXO {
        transaction_id: "funding-tx".to_string(),
        output_index: 1,
        amount: 3_000,
        recipient_address: "sender-address".to_string(),
    };
    let input = TxInput {
        utxo_id: get_utxo_id(&source_utxo.transaction_id, source_utxo.output_index),
        utxo_output_index: source_utxo.output_index,
        signature: Vec::new(),
        sender_address: "sender-address".to_string(),
    };
    let outputs = vec![
        TxOutput {
            amount: 2_000,
            recipient_address: "receiver-address".to_string(),
        },
        TxOutput {
            amount: 1_000,
            recipient_address: "sender-address".to_string(),
        },
    ];
    let tx = Transaction::new(vec![input], outputs);

    assert_eq!(tx.get_total_input_amount(&[source_utxo]), 3_000);
    assert_eq!(tx.get_total_output_amount(), 3_000);
}
