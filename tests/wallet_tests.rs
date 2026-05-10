use std::collections::HashMap;

use blockchain_sim::transaction::{OutPoint, UTXO};
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
        outpoint: OutPoint {
            txid: "genesis-tx".to_string(),
            vout: 0,
        },
        amount: 5_000,
        recipient_address: owner_address,
    });

    let tx = wallet
        .create_transaction("receiver-address", 3_000)
        .expect("Islem olusmali");

    assert_eq!(tx.inputs.len(), 1);
    assert_eq!(tx.outputs.len(), 2);
    assert_eq!(tx.outputs[0].amount, 3_000);
    assert_eq!(tx.outputs[1].amount, 1_000);
}

#[test]
fn rebuild_from_utxo_set_adresi_degistirmeden_statei_guncellemeli() {
    let mut wallet = Wallet::new();
    let owner_address = wallet.get_address().to_string();
    let before_address = wallet.get_address().to_string();

    let mut global_utxo_set = HashMap::new();
    global_utxo_set.insert(
        OutPoint {
            txid: "owner-utxo".to_string(),
            vout: 0,
        },
        UTXO {
            outpoint: OutPoint {
                txid: "owner-utxo".to_string(),
                vout: 0,
            },
            amount: 7_000,
            recipient_address: owner_address,
        },
    );
    global_utxo_set.insert(
        OutPoint {
            txid: "other-utxo".to_string(),
            vout: 1,
        },
        UTXO {
            outpoint: OutPoint {
                txid: "other-utxo".to_string(),
                vout: 1,
            },
            amount: 9_000,
            recipient_address: "other-address".to_string(),
        },
    );

    wallet.rebuild_from_utxo_set(&global_utxo_set);

    assert_eq!(wallet.get_address(), before_address);
    assert_eq!(wallet.get_balance(), 7_000);
}
