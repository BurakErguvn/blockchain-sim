use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::Path;

use crate::network::BlockchainNetwork;

#[derive(Debug, Parser)]
#[command(
    name = "sim-cli",
    version,
    about = "Blockchain-sim için modern komut satırı aracı"
)]
pub struct Cli {
    #[arg(
        long,
        default_value = BlockchainNetwork::DEFAULT_STATE_PATH,
        global = true
    )]
    pub state_path: String,
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init(InitArgs),
    Status,
    Nodes {
        #[command(subcommand)]
        command: NodesCommand,
    },
    Chain {
        #[command(subcommand)]
        command: ChainCommand,
    },
    Tx {
        #[command(subcommand)]
        command: TxCommand,
    },
    Mempool {
        #[command(subcommand)]
        command: MempoolCommand,
    },
    Mine {
        #[command(subcommand)]
        command: MineCommand,
    },
    Persistence {
        #[command(subcommand)]
        command: PersistenceCommand,
    },
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long, default_value_t = 5)]
    pub nodes: usize,
    #[arg(long, default_value_t = 2)]
    pub difficulty: usize,
    #[arg(long, default_value_t = 60)]
    pub block_time: u64,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Subcommand)]
pub enum NodesCommand {
    List,
    Show { id: usize },
}

#[derive(Debug, Subcommand)]
pub enum ChainCommand {
    Tip,
    Show {
        #[arg(long)]
        node_id: usize,
        #[arg(long)]
        limit: Option<usize>,
    },
}

#[derive(Debug, Subcommand)]
pub enum TxCommand {
    Create {
        #[arg(long)]
        sender_id: usize,
        #[arg(long)]
        recipient_id: usize,
        #[arg(long)]
        amount_coin: f64,
    },
}

#[derive(Debug, Subcommand)]
pub enum MempoolCommand {
    List,
}

#[derive(Debug, Subcommand)]
pub enum MineCommand {
    Once,
}

#[derive(Debug, Subcommand)]
pub enum PersistenceCommand {
    Info,
    Save {
        #[arg(long)]
        path: Option<String>,
    },
    Load {
        #[arg(long)]
        path: String,
    },
}

#[derive(Serialize)]
struct StatusView {
    node_count: usize,
    current_validator_id: Option<usize>,
    difficulty: usize,
    block_time: u64,
    mempool_count: usize,
    tip_height: Option<usize>,
    tip_hash: Option<String>,
}

#[derive(Serialize)]
struct NodeView {
    id: usize,
    address: String,
    balance: u64,
    connections: Vec<usize>,
    blockchain_len: usize,
    is_validator: bool,
    utxo_count: usize,
}

#[derive(Serialize)]
struct ChainBlockView {
    index: usize,
    hash: String,
    previous_hash: String,
    timestamp: u64,
    tx_count: usize,
}

#[derive(Serialize)]
struct MempoolView {
    id: String,
    sender: String,
    output_count: usize,
    total_output_amount: u64,
    estimated_size_bytes: usize,
    fee_rate_sat_per_kb: Option<u64>,
}

#[derive(Serialize)]
struct ActionResult {
    message: String,
}

fn coin_to_satoshi(amount_coin: f64) -> Result<u64, String> {
    if !amount_coin.is_finite() || amount_coin <= 0.0 {
        return Err("amount_coin sıfırdan büyük olmalı".to_string());
    }
    let satoshi = (amount_coin * 100_000_000.0).round();
    if satoshi <= 0.0 {
        return Err("amount_coin çok küçük".to_string());
    }
    Ok(satoshi as u64)
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let output = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    println!("{}", output);
    Ok(())
}

fn bootstrap_network(args: &InitArgs) -> Result<BlockchainNetwork, String> {
    if args.nodes == 0 {
        return Err("En az 1 node gerekli".to_string());
    }

    let mut network = BlockchainNetwork::new();
    network.set_difficulty(args.difficulty);
    network.set_block_time(args.block_time);

    for _ in 0..args.nodes {
        network.add_node();
    }
    for i in 0..network.node_count() {
        for j in (i + 1)..network.node_count() {
            network.connect_nodes(i, j);
        }
    }

    network.select_random_validator();
    let Some(_) = network.mine_block() else {
        return Err("Genesis bloğu üretilemedi".to_string());
    };

    Ok(network)
}

fn load_state(path: &str) -> Result<BlockchainNetwork, String> {
    BlockchainNetwork::load_from_disk(path)
        .map_err(|err| format!("State yüklenemedi ({}): {}", path, err))
}

fn save_state(network: &BlockchainNetwork, path: &str) -> Result<(), String> {
    network
        .save_to_disk(path)
        .map_err(|err| format!("State kaydedilemedi ({}): {}", path, err))
}

pub fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Init(args) => {
            let state_path = Path::new(&cli.state_path);
            if state_path.exists() && !args.force {
                return Err(format!(
                    "{} zaten mevcut. Üzerine yazmak için --force kullanın.",
                    cli.state_path
                ));
            }

            let network = bootstrap_network(&args)?;
            save_state(&network, &cli.state_path)?;

            let message = ActionResult {
                message: format!(
                    "Ağ hazırlandı: {} node, difficulty {}, block_time {}s",
                    args.nodes, args.difficulty, args.block_time
                ),
            };
            if cli.json {
                print_json(&message)?;
            } else {
                println!("{}", message.message);
            }
            Ok(())
        }
        Command::Status => {
            let network = load_state(&cli.state_path)?;
            let tip = network
                .nodes
                .first()
                .and_then(|node| node.blockchain.last())
                .map(|block| (block.index, block.hash.clone()));
            let status = StatusView {
                node_count: network.node_count(),
                current_validator_id: network.current_val_id(),
                difficulty: network.difficulty,
                block_time: network.block_time,
                mempool_count: network.mempool.len(),
                tip_height: tip.as_ref().map(|(height, _)| *height),
                tip_hash: tip.as_ref().map(|(_, hash)| hash.clone()),
            };

            if cli.json {
                print_json(&status)?;
            } else {
                println!("Node sayısı        : {}", status.node_count);
                println!("Aktif validator    : {:?}", status.current_validator_id);
                println!("Zorluk             : {}", status.difficulty);
                println!("Blok süresi        : {} sn", status.block_time);
                println!("Mempool tx sayısı  : {}", status.mempool_count);
                println!("Tip yükseklik      : {:?}", status.tip_height);
                println!("Tip hash           : {:?}", status.tip_hash);
            }
            Ok(())
        }
        Command::Nodes { command } => {
            let network = load_state(&cli.state_path)?;
            match command {
                NodesCommand::List => {
                    let nodes: Vec<NodeView> = network
                        .nodes
                        .iter()
                        .map(|node| NodeView {
                            id: node.id,
                            address: node.wallet.get_address().to_string(),
                            balance: node.wallet.get_balance(),
                            connections: node.connections.clone(),
                            blockchain_len: node.blockchain.len(),
                            is_validator: node.is_validator,
                            utxo_count: node.utxo_set.len(),
                        })
                        .collect();

                    if cli.json {
                        print_json(&nodes)?;
                    } else {
                        for node in &nodes {
                            println!(
                                "#{} {} | balance={} | chain={} | validator={}",
                                node.id,
                                node.address,
                                node.balance,
                                node.blockchain_len,
                                node.is_validator
                            );
                        }
                    }
                    Ok(())
                }
                NodesCommand::Show { id } => {
                    let Some(node) = network.nodes.get(id) else {
                        return Err(format!("Node bulunamadı: {}", id));
                    };
                    let view = NodeView {
                        id: node.id,
                        address: node.wallet.get_address().to_string(),
                        balance: node.wallet.get_balance(),
                        connections: node.connections.clone(),
                        blockchain_len: node.blockchain.len(),
                        is_validator: node.is_validator,
                        utxo_count: node.utxo_set.len(),
                    };
                    if cli.json {
                        print_json(&view)?;
                    } else {
                        println!("Node #{}", view.id);
                        println!("  address     : {}", view.address);
                        println!("  balance     : {}", view.balance);
                        println!("  chain_len   : {}", view.blockchain_len);
                        println!("  validator   : {}", view.is_validator);
                        println!("  utxo_count  : {}", view.utxo_count);
                        println!("  connections : {:?}", view.connections);
                    }
                    Ok(())
                }
            }
        }
        Command::Chain { command } => {
            let network = load_state(&cli.state_path)?;
            match command {
                ChainCommand::Tip => {
                    let Some(node) = network.nodes.first() else {
                        return Err("Ağda node yok".to_string());
                    };
                    let Some(tip) = node.blockchain.last() else {
                        return Err("Zincir boş".to_string());
                    };
                    let view = ChainBlockView {
                        index: tip.index,
                        hash: tip.hash.clone(),
                        previous_hash: tip.previous_hash.clone(),
                        timestamp: tip.timestamp,
                        tx_count: tip.transactions.len(),
                    };
                    if cli.json {
                        print_json(&view)?;
                    } else {
                        println!("Tip index : {}", view.index);
                        println!("Tip hash  : {}", view.hash);
                        println!("Prev hash : {}", view.previous_hash);
                        println!("Tx count  : {}", view.tx_count);
                    }
                    Ok(())
                }
                ChainCommand::Show { node_id, limit } => {
                    let Some(node) = network.nodes.get(node_id) else {
                        return Err(format!("Node bulunamadı: {}", node_id));
                    };

                    let blocks_iter = node.blockchain.iter();
                    let views: Vec<ChainBlockView> = if let Some(limit) = limit {
                        blocks_iter
                            .rev()
                            .take(limit)
                            .map(|block| ChainBlockView {
                                index: block.index,
                                hash: block.hash.clone(),
                                previous_hash: block.previous_hash.clone(),
                                timestamp: block.timestamp,
                                tx_count: block.transactions.len(),
                            })
                            .collect()
                    } else {
                        blocks_iter
                            .map(|block| ChainBlockView {
                                index: block.index,
                                hash: block.hash.clone(),
                                previous_hash: block.previous_hash.clone(),
                                timestamp: block.timestamp,
                                tx_count: block.transactions.len(),
                            })
                            .collect()
                    };

                    if cli.json {
                        print_json(&views)?;
                    } else {
                        for block in &views {
                            println!(
                                "#{} {} tx={} prev={}",
                                block.index, block.hash, block.tx_count, block.previous_hash
                            );
                        }
                    }
                    Ok(())
                }
            }
        }
        Command::Tx { command } => match command {
            TxCommand::Create {
                sender_id,
                recipient_id,
                amount_coin,
            } => {
                let mut network = load_state(&cli.state_path)?;
                if sender_id >= network.node_count() || recipient_id >= network.node_count() {
                    return Err("Geçersiz sender_id veya recipient_id".to_string());
                }
                let amount_satoshi = coin_to_satoshi(amount_coin)?;
                let recipient_address = network.get_node_address(recipient_id);
                let Some(tx) = network.create_transaction(sender_id, &recipient_address, amount_satoshi)
                else {
                    return Err("İşlem oluşturulamadı".to_string());
                };
                save_state(&network, &cli.state_path)?;

                #[derive(Serialize)]
                struct TxCreateResult {
                    tx_id: String,
                    sender_id: usize,
                    recipient_id: usize,
                    amount_satoshi: u64,
                    mempool_count: usize,
                }
                let result = TxCreateResult {
                    tx_id: tx.id,
                    sender_id,
                    recipient_id,
                    amount_satoshi,
                    mempool_count: network.mempool.len(),
                };
                if cli.json {
                    print_json(&result)?;
                } else {
                    println!(
                        "İşlem oluşturuldu: {} | mempool={} ",
                        result.tx_id, result.mempool_count
                    );
                }
                Ok(())
            }
        },
        Command::Mempool { command } => match command {
            MempoolCommand::List => {
                let network = load_state(&cli.state_path)?;
                let fee_ref = network.nodes.first().map(|node| &node.utxo_set);
                let entries: Vec<MempoolView> = network
                    .mempool
                    .iter()
                    .map(|tx| MempoolView {
                        id: tx.id.clone(),
                        sender: tx
                            .inputs
                            .first()
                            .map(|input| input.sender_address.clone())
                            .unwrap_or_else(|| "COINBASE".to_string()),
                        output_count: tx.outputs.len(),
                        total_output_amount: tx.get_total_output_amount(),
                        estimated_size_bytes: tx.estimated_size_bytes(),
                        fee_rate_sat_per_kb: fee_ref
                            .and_then(|utxo_set| tx.calculate_fee_rate_sat_per_kb(utxo_set)),
                    })
                    .collect();

                if cli.json {
                    print_json(&entries)?;
                } else if entries.is_empty() {
                    println!("Mempool boş");
                } else {
                    for entry in &entries {
                        println!(
                            "{} | sender={} | outputs={} | amount={} | feerate={:?}",
                            entry.id,
                            entry.sender,
                            entry.output_count,
                            entry.total_output_amount,
                            entry.fee_rate_sat_per_kb
                        );
                    }
                }
                Ok(())
            }
        },
        Command::Mine { command } => match command {
            MineCommand::Once => {
                let mut network = load_state(&cli.state_path)?;
                if network.current_val_id().is_none() {
                    network.select_random_validator();
                }
                let Some(block) = network.mine_block() else {
                    return Err("Blok üretilemedi".to_string());
                };
                save_state(&network, &cli.state_path)?;

                let result = ChainBlockView {
                    index: block.index,
                    hash: block.hash,
                    previous_hash: block.previous_hash,
                    timestamp: block.timestamp,
                    tx_count: block.transactions.len(),
                };
                if cli.json {
                    print_json(&result)?;
                } else {
                    println!(
                        "Blok üretildi: #{} {} (tx={})",
                        result.index, result.hash, result.tx_count
                    );
                }
                Ok(())
            }
        },
        Command::Persistence { command } => match command {
            PersistenceCommand::Info => {
                let exists = Path::new(&cli.state_path).exists();
                #[derive(Serialize)]
                struct InfoResult {
                    state_path: String,
                    exists: bool,
                }
                let result = InfoResult {
                    state_path: cli.state_path,
                    exists,
                };
                if cli.json {
                    print_json(&result)?;
                } else {
                    println!("state_path: {}", result.state_path);
                    println!("exists    : {}", result.exists);
                }
                Ok(())
            }
            PersistenceCommand::Save { path } => {
                let target_path = path.unwrap_or(cli.state_path.clone());
                let network = load_state(&cli.state_path)?;
                save_state(&network, &target_path)?;

                let result = ActionResult {
                    message: format!("State kaydedildi: {}", target_path),
                };
                if cli.json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            PersistenceCommand::Load { path } => {
                let network = load_state(&path)?;
                save_state(&network, &cli.state_path)?;
                let result = ActionResult {
                    message: format!("State yüklendi: {} -> {}", path, cli.state_path),
                };
                if cli.json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::coin_to_satoshi;

    #[test]
    fn coin_to_satoshi_pozitif_degerde_calismali() {
        assert_eq!(coin_to_satoshi(1.25).expect("donusmeli"), 125_000_000);
    }

    #[test]
    fn coin_to_satoshi_sifir_ve_negatifte_hata_vermeli() {
        assert!(coin_to_satoshi(0.0).is_err());
        assert!(coin_to_satoshi(-1.0).is_err());
    }
}
