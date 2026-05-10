use clap::{Args, Parser, Subcommand};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use strsim::jaro_winkler;

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
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Repl,
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
    Scenario {
        #[command(subcommand)]
        command: ScenarioCommand,
    },
    Alias {
        #[command(subcommand)]
        command: AliasCommand,
    },
    Macro {
        #[command(subcommand)]
        command: MacroCommand,
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

#[derive(Debug, Subcommand)]
pub enum ScenarioCommand {
    List,
    Run {
        name: String,
        #[arg(long, default_value_t = 0)]
        primary: usize,
        #[arg(long, default_value_t = 1)]
        secondary: usize,
    },
}

#[derive(Debug, Subcommand)]
pub enum AliasCommand {
    List,
    Add {
        name: String,
        #[arg(required = true, num_args = 1..)]
        expansion: Vec<String>,
    },
    Remove {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum MacroCommand {
    List,
    Add {
        name: String,
        #[arg(long = "cmd", required = true, num_args = 1..)]
        commands: Vec<String>,
    },
    Remove {
        name: String,
    },
    Run {
        name: String,
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

#[derive(Debug, Serialize, Deserialize, Default)]
struct AliasStore {
    aliases: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MacroStore {
    macros: BTreeMap<String, Vec<String>>,
}

const ROOT_COMMANDS: &[&str] = &[
    "repl",
    "init",
    "status",
    "nodes",
    "chain",
    "tx",
    "mempool",
    "mine",
    "persistence",
    "scenario",
    "alias",
    "macro",
    "help",
    "exit",
    "quit",
];
const MAX_MACRO_DEPTH: usize = 5;

struct CliReplHelper {
    alias_names: Vec<String>,
    hinter: HistoryHinter,
}

impl CliReplHelper {
    fn new(alias_names: Vec<String>) -> Self {
        Self {
            alias_names,
            hinter: HistoryHinter::new(),
        }
    }
}

impl Helper for CliReplHelper {}
impl Validator for CliReplHelper {}
impl Highlighter for CliReplHelper {}

impl Hinter for CliReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Completer for CliReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let input = &line[..pos];
        let cursor_start = input
            .rfind(char::is_whitespace)
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let needle = &input[cursor_start..];
        let candidates = completion_candidates(input, &self.alias_names);
        let matches = candidates
            .into_iter()
            .filter(|candidate| candidate.starts_with(needle))
            .map(|candidate| Pair {
                display: candidate.clone(),
                replacement: candidate,
            })
            .collect();

        Ok((cursor_start, matches))
    }
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

fn alias_file_path(state_path: &str) -> String {
    let state_path = Path::new(state_path);
    let base_dir = state_path.parent().unwrap_or_else(|| Path::new("."));
    base_dir
        .join(".sim_cli_aliases.json")
        .to_string_lossy()
        .to_string()
}

fn macro_file_path(state_path: &str) -> String {
    let state_path = Path::new(state_path);
    let base_dir = state_path.parent().unwrap_or_else(|| Path::new("."));
    base_dir
        .join(".sim_cli_macros.json")
        .to_string_lossy()
        .to_string()
}

fn load_alias_store(state_path: &str) -> Result<AliasStore, String> {
    let file_path = alias_file_path(state_path);
    let path = Path::new(&file_path);
    if !path.exists() {
        return Ok(AliasStore::default());
    }
    let content = fs::read(path).map_err(|err| err.to_string())?;
    serde_json::from_slice(&content).map_err(|err| err.to_string())
}

fn save_alias_store(state_path: &str, aliases: &AliasStore) -> Result<(), String> {
    let file_path = alias_file_path(state_path);
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let content = serde_json::to_vec_pretty(aliases).map_err(|err| err.to_string())?;
    fs::write(file_path, content).map_err(|err| err.to_string())
}

fn load_macro_store(state_path: &str) -> Result<MacroStore, String> {
    let file_path = macro_file_path(state_path);
    let path = Path::new(&file_path);
    if !path.exists() {
        return Ok(MacroStore::default());
    }
    let content = fs::read(path).map_err(|err| err.to_string())?;
    serde_json::from_slice(&content).map_err(|err| err.to_string())
}

fn save_macro_store(state_path: &str, macros: &MacroStore) -> Result<(), String> {
    let file_path = macro_file_path(state_path);
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let content = serde_json::to_vec_pretty(macros).map_err(|err| err.to_string())?;
    fs::write(file_path, content).map_err(|err| err.to_string())
}

fn namespaced_subcommands(root: &str) -> &'static [&'static str] {
    match root {
        "nodes" => &["list", "show"],
        "chain" => &["tip", "show"],
        "tx" => &["create"],
        "mempool" => &["list"],
        "mine" => &["once"],
        "persistence" => &["info", "save", "load"],
        "scenario" => &["list", "run"],
        "alias" => &["list", "add", "remove"],
        "macro" => &["list", "add", "remove", "run"],
        _ => &[],
    }
}

fn completion_candidates(input: &str, alias_names: &[String]) -> Vec<String> {
    let has_trailing_space = input.chars().last().is_some_and(char::is_whitespace);
    let mut tokens: Vec<&str> = input.split_whitespace().collect();
    if has_trailing_space {
        tokens.push("");
    }

    if tokens.is_empty() || tokens[0].is_empty() {
        let mut commands: Vec<String> = ROOT_COMMANDS.iter().map(|s| (*s).to_string()).collect();
        commands.extend(alias_names.iter().cloned());
        commands.sort();
        commands.dedup();
        return commands;
    }

    if tokens.len() == 1 {
        let mut commands: Vec<String> = ROOT_COMMANDS.iter().map(|s| (*s).to_string()).collect();
        commands.extend(alias_names.iter().cloned());
        commands.sort();
        commands.dedup();
        return commands;
    }

    namespaced_subcommands(tokens[0])
        .iter()
        .map(|command| (*command).to_string())
        .collect()
}

fn history_file_path(state_path: &str) -> String {
    let state_path = Path::new(state_path);
    let base_dir = state_path.parent().unwrap_or_else(|| Path::new("."));
    base_dir
        .join(".sim_cli_history")
        .to_string_lossy()
        .to_string()
}

fn print_repl_help() {
    println!(
        "Komutlar: init, status, nodes, chain, tx, mempool, mine, persistence, scenario, alias, macro"
    );
    println!("Macro kısayolu: !<macro_adi>");
    println!("Yardım: help");
    println!("Çıkış: exit | quit");
}

fn suggest_root_commands(token: &str, alias_names: &[String]) -> Vec<String> {
    let mut pool: Vec<String> = ROOT_COMMANDS
        .iter()
        .map(|command| (*command).to_string())
        .collect();
    pool.extend(alias_names.iter().cloned());
    pool.sort();
    pool.dedup();

    let mut candidates: Vec<(f64, String)> = pool
        .iter()
        .filter_map(|command| {
            let score = jaro_winkler(command, token);
            if score >= 0.75 {
                Some((score, command.to_string()))
            } else {
                None
            }
        })
        .collect();
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
    candidates.into_iter().map(|(_, command)| command).collect()
}

fn expand_alias_tokens(
    tokens: Vec<String>,
    alias_store: &AliasStore,
) -> Result<Vec<String>, String> {
    if let Some(first) = tokens.first() {
        if let Some(expansion) = alias_store.aliases.get(first) {
            let mut expanded = shell_words::split(expansion).map_err(|err| err.to_string())?;
            expanded.extend(tokens.into_iter().skip(1));
            return Ok(expanded);
        }
    }
    Ok(tokens)
}

fn execute_command(
    state_path: &str,
    json: bool,
    command: Command,
    macro_depth: usize,
) -> Result<(), String> {
    match command {
        Command::Repl => run_repl(state_path, json),
        Command::Init(args) => {
            let state_path_ref = Path::new(state_path);
            if state_path_ref.exists() && !args.force {
                return Err(format!(
                    "{} zaten mevcut. Üzerine yazmak için --force kullanın.",
                    state_path
                ));
            }

            let network = bootstrap_network(&args)?;
            save_state(&network, state_path)?;

            let message = ActionResult {
                message: format!(
                    "Ağ hazırlandı: {} node, difficulty {}, block_time {}s",
                    args.nodes, args.difficulty, args.block_time
                ),
            };
            if json {
                print_json(&message)?;
            } else {
                println!("{}", message.message);
            }
            Ok(())
        }
        Command::Status => {
            let network = load_state(state_path)?;
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

            if json {
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
            let network = load_state(state_path)?;
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

                    if json {
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
                    if json {
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
            let network = load_state(state_path)?;
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
                    if json {
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

                    if json {
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
                let mut network = load_state(state_path)?;
                if sender_id >= network.node_count() || recipient_id >= network.node_count() {
                    return Err("Geçersiz sender_id veya recipient_id".to_string());
                }
                let amount_satoshi = coin_to_satoshi(amount_coin)?;
                let recipient_address = network.get_node_address(recipient_id);
                let Some(tx) =
                    network.create_transaction(sender_id, &recipient_address, amount_satoshi)
                else {
                    return Err("İşlem oluşturulamadı".to_string());
                };
                save_state(&network, state_path)?;

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
                if json {
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
                let network = load_state(state_path)?;
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

                if json {
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
                let mut network = load_state(state_path)?;
                if network.current_val_id().is_none() {
                    network.select_random_validator();
                }
                let Some(block) = network.mine_block() else {
                    return Err("Blok üretilemedi".to_string());
                };
                save_state(&network, state_path)?;

                let result = ChainBlockView {
                    index: block.index,
                    hash: block.hash,
                    previous_hash: block.previous_hash,
                    timestamp: block.timestamp,
                    tx_count: block.transactions.len(),
                };
                if json {
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
                let exists = Path::new(state_path).exists();
                #[derive(Serialize)]
                struct InfoResult {
                    state_path: String,
                    exists: bool,
                }
                let result = InfoResult {
                    state_path: state_path.to_string(),
                    exists,
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("state_path: {}", result.state_path);
                    println!("exists    : {}", result.exists);
                }
                Ok(())
            }
            PersistenceCommand::Save { path } => {
                let target_path = path.unwrap_or_else(|| state_path.to_string());
                let network = load_state(state_path)?;
                save_state(&network, &target_path)?;

                let result = ActionResult {
                    message: format!("State kaydedildi: {}", target_path),
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            PersistenceCommand::Load { path } => {
                let network = load_state(&path)?;
                save_state(&network, state_path)?;
                let result = ActionResult {
                    message: format!("State yüklendi: {} -> {}", path, state_path),
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
        },
        Command::Scenario { command } => match command {
            ScenarioCommand::List => {
                #[derive(Serialize)]
                struct ScenarioList {
                    scenarios: Vec<&'static str>,
                }
                let result = ScenarioList {
                    scenarios: vec!["quickstart", "fork-reorg"],
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("Kullanılabilir senaryolar:");
                    for scenario in &result.scenarios {
                        println!("- {}", scenario);
                    }
                }
                Ok(())
            }
            ScenarioCommand::Run {
                name,
                primary,
                secondary,
            } => match name.as_str() {
                "quickstart" => {
                    let mut network = if Path::new(state_path).exists() {
                        load_state(state_path)?
                    } else {
                        bootstrap_network(&InitArgs {
                            nodes: 5,
                            difficulty: 2,
                            block_time: 60,
                            force: true,
                        })?
                    };
                    if network.current_val_id().is_none() {
                        network.select_random_validator();
                    }
                    let _ = network
                        .mine_block()
                        .ok_or("Senaryo için blok üretilemedi")?;
                    save_state(&network, state_path)?;
                    let result = ActionResult {
                        message: "quickstart senaryosu tamamlandı".to_string(),
                    };
                    if json {
                        print_json(&result)?;
                    } else {
                        println!("{}", result.message);
                    }
                    Ok(())
                }
                "fork-reorg" => {
                    let mut network = load_state(state_path)?;
                    let reorg_depth = network
                        .simulate_fork_and_reorg(primary, secondary)
                        .map_err(|err| format!("Senaryo hatası: {}", err))?;
                    save_state(&network, state_path)?;
                    #[derive(Serialize)]
                    struct ScenarioRunResult {
                        scenario: String,
                        reorg_depth: usize,
                    }
                    let result = ScenarioRunResult {
                        scenario: name,
                        reorg_depth,
                    };
                    if json {
                        print_json(&result)?;
                    } else {
                        println!("fork-reorg tamamlandı. Reorg depth: {}", result.reorg_depth);
                    }
                    Ok(())
                }
                _ => Err(format!("Bilinmeyen senaryo: {}", name)),
            },
        },
        Command::Alias { command } => match command {
            AliasCommand::List => {
                let alias_store = load_alias_store(state_path)?;
                if json {
                    print_json(&alias_store.aliases)?;
                } else if alias_store.aliases.is_empty() {
                    println!("Alias tanımı yok.");
                } else {
                    for (name, expansion) in alias_store.aliases {
                        println!("{} -> {}", name, expansion);
                    }
                }
                Ok(())
            }
            AliasCommand::Add { name, expansion } => {
                if ROOT_COMMANDS.iter().any(|command| *command == name) {
                    return Err("Alias adı yerleşik komutlarla çakışamaz".to_string());
                }
                let mut alias_store = load_alias_store(state_path)?;
                alias_store
                    .aliases
                    .insert(name.clone(), expansion.join(" "));
                save_alias_store(state_path, &alias_store)?;
                let result = ActionResult {
                    message: format!("Alias eklendi: {}", name),
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            AliasCommand::Remove { name } => {
                let mut alias_store = load_alias_store(state_path)?;
                if alias_store.aliases.remove(&name).is_none() {
                    return Err(format!("Alias bulunamadı: {}", name));
                }
                save_alias_store(state_path, &alias_store)?;
                let result = ActionResult {
                    message: format!("Alias silindi: {}", name),
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
        },
        Command::Macro { command } => match command {
            MacroCommand::List => {
                let macros = load_macro_store(state_path)?;
                if json {
                    print_json(&macros.macros)?;
                } else if macros.macros.is_empty() {
                    println!("Macro tanımı yok.");
                } else {
                    for (name, commands) in macros.macros {
                        println!("{}:", name);
                        for command in commands {
                            println!("  - {}", command);
                        }
                    }
                }
                Ok(())
            }
            MacroCommand::Add { name, commands } => {
                if commands.is_empty() {
                    return Err("Macro en az bir komut içermeli".to_string());
                }
                let mut macros = load_macro_store(state_path)?;
                macros.macros.insert(name.clone(), commands);
                save_macro_store(state_path, &macros)?;
                let result = ActionResult {
                    message: format!("Macro eklendi: {}", name),
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            MacroCommand::Remove { name } => {
                let mut macros = load_macro_store(state_path)?;
                if macros.macros.remove(&name).is_none() {
                    return Err(format!("Macro bulunamadı: {}", name));
                }
                save_macro_store(state_path, &macros)?;
                let result = ActionResult {
                    message: format!("Macro silindi: {}", name),
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
            MacroCommand::Run { name } => {
                if macro_depth >= MAX_MACRO_DEPTH {
                    return Err("Maksimum macro çağrı derinliğine ulaşıldı".to_string());
                }
                let macros = load_macro_store(state_path)?;
                let Some(commands) = macros.macros.get(&name) else {
                    return Err(format!("Macro bulunamadı: {}", name));
                };
                for command_line in commands {
                    let mut tokens =
                        shell_words::split(command_line).map_err(|err| err.to_string())?;
                    let alias_store = load_alias_store(state_path)?;
                    tokens = expand_alias_tokens(tokens, &alias_store)?;
                    let mut args = vec![
                        "sim-cli".to_string(),
                        "--state-path".to_string(),
                        state_path.to_string(),
                    ];
                    if json {
                        args.push("--json".to_string());
                    }
                    args.extend(tokens);

                    let parsed = Cli::try_parse_from(args).map_err(|err| err.to_string())?;
                    let Some(command) = parsed.command else {
                        continue;
                    };
                    execute_command(&parsed.state_path, parsed.json, command, macro_depth + 1)?;
                }
                let result = ActionResult {
                    message: format!("Macro çalıştırıldı: {}", name),
                };
                if json {
                    print_json(&result)?;
                } else {
                    println!("{}", result.message);
                }
                Ok(())
            }
        },
    }
}

fn run_repl(state_path: &str, json: bool) -> Result<(), String> {
    let history_path = history_file_path(state_path);
    if let Some(parent) = Path::new(&history_path).parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let initial_alias_store = load_alias_store(state_path).unwrap_or_default();
    let alias_names: Vec<String> = initial_alias_store.aliases.keys().cloned().collect();
    let mut editor: Editor<CliReplHelper, DefaultHistory> =
        Editor::new().map_err(|err| err.to_string())?;
    editor.set_helper(Some(CliReplHelper::new(alias_names)));
    let _ = editor.load_history(&history_path);

    println!("sim-cli repl başlatıldı. Yardım için 'help', çıkış için 'exit'.");
    loop {
        match editor.readline("sim> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let _ = editor.add_history_entry(line);
                if line == "exit" || line == "quit" {
                    break;
                }
                if line == "help" {
                    print_repl_help();
                    continue;
                }

                let mut tokens = match shell_words::split(line) {
                    Ok(tokens) => tokens,
                    Err(err) => {
                        println!("Parse hatası: {}", err);
                        continue;
                    }
                };

                if let Some(first) = tokens.first_mut() {
                    if first.starts_with('!') && first.len() > 1 {
                        let macro_name = first.trim_start_matches('!').to_string();
                        tokens = vec!["macro".to_string(), "run".to_string(), macro_name];
                    }
                }

                let alias_store = load_alias_store(state_path).unwrap_or_default();
                let alias_names: Vec<String> = alias_store.aliases.keys().cloned().collect();
                tokens = match expand_alias_tokens(tokens, &alias_store) {
                    Ok(tokens) => tokens,
                    Err(err) => {
                        println!("Alias açılım hatası: {}", err);
                        continue;
                    }
                };

                let mut args = vec![
                    "sim-cli".to_string(),
                    "--state-path".to_string(),
                    state_path.to_string(),
                ];
                if json {
                    args.push("--json".to_string());
                }
                args.extend(tokens.clone());

                match Cli::try_parse_from(args) {
                    Ok(parsed) => match parsed.command {
                        Some(Command::Repl) => {
                            println!("Zaten REPL modundasın.");
                        }
                        Some(command) => {
                            if let Err(err) =
                                execute_command(&parsed.state_path, parsed.json, command, 0)
                            {
                                println!("Hata: {}", err);
                            }
                            let refreshed_alias_store =
                                load_alias_store(state_path).unwrap_or_default();
                            let refreshed_alias_names =
                                refreshed_alias_store.aliases.keys().cloned().collect();
                            editor.set_helper(Some(CliReplHelper::new(refreshed_alias_names)));
                        }
                        None => print_repl_help(),
                    },
                    Err(err) => {
                        println!("{}", err);
                        if let Some(first) = tokens.first() {
                            let suggestions = suggest_root_commands(first, &alias_names);
                            if !suggestions.is_empty() {
                                println!("Öneri: {}", suggestions.join(", "));
                            }
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => return Err(format!("REPL hatası: {}", err)),
        }
    }

    let _ = editor.save_history(&history_path);
    Ok(())
}

pub fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Some(command) => execute_command(&cli.state_path, cli.json, command, 0),
        None => run_repl(&cli.state_path, cli.json),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        coin_to_satoshi, completion_candidates, expand_alias_tokens, suggest_root_commands,
        AliasStore,
    };
    use std::collections::BTreeMap;

    #[test]
    fn coin_to_satoshi_pozitif_degerde_calismali() {
        assert_eq!(coin_to_satoshi(1.25).expect("donusmeli"), 125_000_000);
    }

    #[test]
    fn coin_to_satoshi_sifir_ve_negatifte_hata_vermeli() {
        assert!(coin_to_satoshi(0.0).is_err());
        assert!(coin_to_satoshi(-1.0).is_err());
    }

    #[test]
    fn alias_genislemesi_ilk_token_uzerinden_calismali() {
        let mut store = AliasStore::default();
        store.aliases.insert("st".to_string(), "status".to_string());
        let tokens = vec!["st".to_string()];
        let expanded = expand_alias_tokens(tokens, &store).expect("alias genislemeli");
        assert_eq!(expanded, vec!["status".to_string()]);
    }

    #[test]
    fn namespaced_completion_alt_komutlari_dondurmeli() {
        let alias_names: Vec<String> = Vec::new();
        let candidates = completion_candidates("nodes ", &alias_names);
        assert!(candidates.contains(&"list".to_string()));
        assert!(candidates.contains(&"show".to_string()));
    }

    #[test]
    fn root_suggestion_aliaslari_da_degerlendirmeli() {
        let mut aliases = BTreeMap::new();
        aliases.insert("st".to_string(), "status".to_string());
        let alias_names: Vec<String> = aliases.keys().cloned().collect();
        let suggestions = suggest_root_commands("st", &alias_names);
        assert!(suggestions.contains(&"st".to_string()));
        assert!(suggestions.contains(&"status".to_string()));
    }
}
