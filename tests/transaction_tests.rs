use blockchain_sim::transaction::{Transaction, TxInput, TxOutput, UTXO};

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
        prev_tx_id: source_utxo.transaction_id.clone(),
        prev_output_index: source_utxo.output_index,
        signature: Vec::new(),
        public_key: Vec::new(),
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
        prev_tx_id: source_utxo.transaction_id.clone(),
        prev_output_index: source_utxo.output_index,
        signature: Vec::new(),
        public_key: Vec::new(),
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

#[test]
fn ayni_utxo_iki_defa_harcanamaz() {
    let source_utxo = UTXO {
        transaction_id: "funding-tx".to_string(),
        output_index: 0,
        amount: 4_000,
        recipient_address: "sender-address".to_string(),
    };
    let input = TxInput {
        prev_tx_id: source_utxo.transaction_id.clone(),
        prev_output_index: source_utxo.output_index,
        signature: vec![1, 2, 3],
        public_key: vec![2; 33],
        sender_address: "sender-address".to_string(),
    };
    let duplicate_input = input.clone();
    let outputs = vec![TxOutput {
        amount: 3_000,
        recipient_address: "receiver-address".to_string(),
    }];

    let tx = Transaction::new(vec![input, duplicate_input], outputs);
    assert!(!tx.is_valid(&[source_utxo]));
}

#[test]
fn fee_input_ve_output_farkindan_hesaplanmali() {
    let source_utxo = UTXO {
        transaction_id: "fee-tx".to_string(),
        output_index: 0,
        amount: 5_000,
        recipient_address: "sender-address".to_string(),
    };
    let input = TxInput {
        prev_tx_id: source_utxo.transaction_id.clone(),
        prev_output_index: source_utxo.output_index,
        signature: vec![1, 2, 3],
        public_key: vec![2; 33],
        sender_address: "sender-address".to_string(),
    };
    let outputs = vec![TxOutput {
        amount: 4_200,
        recipient_address: "receiver-address".to_string(),
    }];

    let tx = Transaction::new(vec![input], outputs);
    assert_eq!(tx.calculate_fee(&[source_utxo]), Some(800));
}
