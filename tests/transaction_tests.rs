use std::collections::HashMap;

use blockchain_sim::transaction::{OutPoint, Transaction, TxInput, TxOutput, UTXO};

fn single_utxo_map(utxo: UTXO) -> HashMap<OutPoint, UTXO> {
    let mut utxo_map = HashMap::new();
    utxo_map.insert(utxo.outpoint.clone(), utxo);
    utxo_map
}

#[test]
fn coinbase_transaction_gecerli_olmali() {
    let tx = Transaction::new_coinbase("receiver-address".to_string(), 5_000);
    let utxo_set: HashMap<OutPoint, UTXO> = HashMap::new();

    assert!(tx.is_valid(&utxo_set));
}

#[test]
fn output_inputtan_buyukse_transaction_gecersiz_olmali() {
    let source_utxo = UTXO {
        outpoint: OutPoint {
            txid: "source-tx".to_string(),
            vout: 0,
        },
        amount: 1_000,
        recipient_address: "sender-address".to_string(),
    };
    let input = TxInput {
        previous_output: source_utxo.outpoint.clone(),
        signature: Vec::new(),
        public_key: Vec::new(),
        sender_address: "sender-address".to_string(),
    };
    let outputs = vec![TxOutput {
        amount: 1_500,
        recipient_address: "receiver-address".to_string(),
    }];

    let tx = Transaction::new(vec![input], outputs);
    assert!(!tx.is_valid(&single_utxo_map(source_utxo)));
}

#[test]
fn input_ve_output_toplamlari_hesaplanmali() {
    let source_utxo = UTXO {
        outpoint: OutPoint {
            txid: "funding-tx".to_string(),
            vout: 1,
        },
        amount: 3_000,
        recipient_address: "sender-address".to_string(),
    };
    let input = TxInput {
        previous_output: source_utxo.outpoint.clone(),
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

    assert_eq!(
        tx.get_total_input_amount(&single_utxo_map(source_utxo)),
        3_000
    );
    assert_eq!(tx.get_total_output_amount(), 3_000);
}

#[test]
fn ayni_utxo_iki_defa_harcanamaz() {
    let source_utxo = UTXO {
        outpoint: OutPoint {
            txid: "funding-tx".to_string(),
            vout: 0,
        },
        amount: 4_000,
        recipient_address: "sender-address".to_string(),
    };
    let input = TxInput {
        previous_output: source_utxo.outpoint.clone(),
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
    assert!(!tx.is_valid(&single_utxo_map(source_utxo)));
}

#[test]
fn fee_input_ve_output_farkindan_hesaplanmali() {
    let source_utxo = UTXO {
        outpoint: OutPoint {
            txid: "fee-tx".to_string(),
            vout: 0,
        },
        amount: 5_000,
        recipient_address: "sender-address".to_string(),
    };
    let input = TxInput {
        previous_output: source_utxo.outpoint.clone(),
        signature: vec![1, 2, 3],
        public_key: vec![2; 33],
        sender_address: "sender-address".to_string(),
    };
    let outputs = vec![TxOutput {
        amount: 4_200,
        recipient_address: "receiver-address".to_string(),
    }];

    let tx = Transaction::new(vec![input], outputs);
    assert_eq!(tx.calculate_fee(&single_utxo_map(source_utxo)), Some(800));
}
