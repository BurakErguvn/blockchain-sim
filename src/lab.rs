use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::network::BlockchainNetwork;

pub const DEFAULT_LABS_ROOT: &str = "labs";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabManifest {
    pub id: String,
    pub title: String,
    pub difficulty: String,
    #[serde(default)]
    pub estimated_minutes: u32,
    #[serde(default)]
    pub objectives: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub setup: LabSetup,
    #[serde(default)]
    pub steps: Vec<LabStep>,
    #[serde(default)]
    pub assertions: Vec<LabAssertion>,
    #[serde(default)]
    pub guided_steps: Vec<GuidedStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidedStep {
    pub id: String,
    pub title: String,
    pub instruction: String,
    #[serde(default)]
    pub concept: String,
    #[serde(default)]
    pub command_template: String,
    #[serde(default)]
    pub hint_levels: Vec<String>,
    #[serde(default)]
    pub feedback_fail: String,
    #[serde(default)]
    pub discussion_prompt: String,
    pub check: GuidedCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuidedCheck {
    NodeCount {
        expected: usize,
    },
    BalanceMin {
        node: usize,
        amount_satoshi: u64,
    },
    BalanceEquals {
        node: usize,
        amount_satoshi: u64,
    },
    MempoolCountMin {
        expected: usize,
    },
    MempoolCountEquals {
        expected: usize,
    },
    ChainHeightMin {
        expected: usize,
    },
    TipHashPresent,
    CanonicalTipsMatch,
    DifficultyEquals {
        expected: usize,
    },
    RunSignatureTamperDemo {
        from: usize,
        to: usize,
        amount_coin: f64,
    },
    RunForkReorgDemo {
        #[serde(default)]
        primary: usize,
        #[serde(default = "default_secondary")]
        secondary: usize,
        #[serde(default = "default_reorg_depth")]
        expected_depth: usize,
    },
    Acknowledge {
        #[serde(default)]
        token: String,
    },
}

fn default_reorg_depth() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabSetup {
    pub nodes: usize,
    pub difficulty: usize,
    pub block_time_seconds: u64,
    #[serde(default = "default_true")]
    pub mine_genesis: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum LabStep {
    CreateTransaction {
        from: usize,
        to: usize,
        amount_coin: f64,
        #[serde(default)]
        fee_satoshi: Option<u64>,
    },
    MineBlock,
    SetDifficulty {
        difficulty: usize,
    },
    SimulateForkReorg {
        #[serde(default)]
        primary: usize,
        #[serde(default = "default_secondary")]
        secondary: usize,
    },
    DemonstrateSignatureTamper {
        from: usize,
        to: usize,
        amount_coin: f64,
    },
    DemonstrateDoubleSpend {
        from: usize,
        to_a: usize,
        to_b: usize,
        amount_coin: f64,
    },
    DemonstrateChainTamper {
        #[serde(default)]
        node_id: usize,
    },
}

fn default_secondary() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LabAssertion {
    NodeCount { expected: usize },
    BalanceEquals { node: usize, amount_satoshi: u64 },
    BalanceMin { node: usize, amount_satoshi: u64 },
    ChainHeightMin { expected: usize },
    MempoolCountMin { expected: usize },
    MempoolCountEquals { expected: usize },
    TipHashPresent,
    DifficultyEquals { expected: usize },
    CanonicalTipsMatch,
    ReorgDepthEquals { expected: usize },
    DemoFlagTrue { key: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct LabSummary {
    pub id: String,
    pub title: String,
    pub difficulty: String,
    pub estimated_minutes: u32,
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DemoFlags {
    pub signature_tamper_rejected: bool,
    pub double_spend_second_rejected: bool,
    pub chain_tamper_invalid: bool,
    pub reorg_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepResult {
    pub action: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssertionResult {
    pub assertion: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabRunReport {
    pub lab_id: String,
    pub passed: bool,
    pub steps: Vec<StepResult>,
    pub checks: Vec<AssertionResult>,
    pub demo: DemoFlags,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabVerifyReport {
    pub lab_id: String,
    pub passed: bool,
    pub checks: Vec<AssertionResult>,
    pub demo: DemoFlags,
}

pub fn labs_root_from(explicit: Option<&str>) -> PathBuf {
    explicit
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LABS_ROOT))
}

pub fn discover_labs(root: &Path) -> Result<Vec<LabSummary>, String> {
    if !root.exists() {
        return Err(format!("Labs dizini bulunamadı: {}", root.display()));
    }

    let mut labs = Vec::new();
    let entries = fs::read_dir(root)
        .map_err(|err| format!("Labs dizini okunamadı ({}): {}", root.display(), err))?;

    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join("lab.toml");
        if !manifest_path.exists() {
            continue;
        }
        let manifest = load_manifest(&manifest_path)?;
        labs.push(LabSummary {
            id: manifest.id,
            title: manifest.title,
            difficulty: manifest.difficulty,
            estimated_minutes: manifest.estimated_minutes,
            path: path.display().to_string(),
        });
    }

    labs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(labs)
}

pub fn load_lab(root: &Path, id: &str) -> Result<(LabManifest, PathBuf), String> {
    let candidate = root.join(id).join("lab.toml");
    if candidate.exists() {
        let manifest = load_manifest(&candidate)?;
        return Ok((manifest, candidate.parent().unwrap().to_path_buf()));
    }

    for summary in discover_labs(root)? {
        if summary.id == id {
            let manifest_path = Path::new(&summary.path).join("lab.toml");
            let manifest = load_manifest(&manifest_path)?;
            return Ok((manifest, PathBuf::from(summary.path)));
        }
    }

    Err(format!("Lab bulunamadı: {}", id))
}

pub fn load_manifest(path: &Path) -> Result<LabManifest, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Lab manifest okunamadı ({}): {}", path.display(), err))?;
    let manifest: LabManifest = toml::from_str(&content)
        .map_err(|err| format!("Lab manifest parse hatası ({}): {}", path.display(), err))?;
    if manifest.id.trim().is_empty() {
        return Err(format!("Lab id boş olamaz ({})", path.display()));
    }
    if manifest.setup.nodes == 0 {
        return Err(format!("Lab setup.nodes sıfır olamaz ({})", path.display()));
    }
    Ok(manifest)
}

pub fn setup_network(setup: &LabSetup) -> Result<BlockchainNetwork, String> {
    let mut network = BlockchainNetwork::new();
    network.set_difficulty(setup.difficulty);
    network.set_block_time(setup.block_time_seconds);

    for _ in 0..setup.nodes {
        network.add_node();
    }
    for i in 0..network.node_count() {
        for j in (i + 1)..network.node_count() {
            network.connect_nodes(i, j);
        }
    }

    if setup.mine_genesis {
        // Deterministic funding: node 0 (Alice) mines genesis and receives the reward.
        network.select_validator(0)?;
        network
            .mine_block()
            .ok_or_else(|| "Genesis bloğu üretilemedi".to_string())?;
    }

    Ok(network)
}

pub fn save_network(network: &BlockchainNetwork, state_path: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(state_path).parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    network
        .save_to_disk(state_path)
        .map_err(|err| format!("State kaydedilemedi ({}): {}", state_path, err))
}

pub fn load_network(state_path: &str) -> Result<BlockchainNetwork, String> {
    BlockchainNetwork::load_from_disk(state_path)
        .map_err(|err| format!("State yüklenemedi ({}): {}", state_path, err))
}

pub fn coin_to_satoshi(amount_coin: f64) -> Result<u64, String> {
    if !amount_coin.is_finite() || amount_coin <= 0.0 {
        return Err("amount_coin sıfırdan büyük olmalı".to_string());
    }
    let satoshi = (amount_coin * 100_000_000.0).round();
    if satoshi <= 0.0 {
        return Err("amount_coin çok küçük".to_string());
    }
    Ok(satoshi as u64)
}

pub fn run_lab(manifest: &LabManifest, state_path: &str) -> Result<LabRunReport, String> {
    let mut network = setup_network(&manifest.setup)?;
    let mut demo = DemoFlags::default();
    let mut steps = Vec::new();

    for step in &manifest.steps {
        let result = apply_step(&mut network, step, &mut demo)?;
        let ok = result.ok;
        steps.push(result);
        if !ok {
            save_network(&network, state_path)?;
            return Ok(LabRunReport {
                lab_id: manifest.id.clone(),
                passed: false,
                steps,
                checks: Vec::new(),
                demo,
            });
        }
    }

    save_network(&network, state_path)?;
    let checks = evaluate_assertions(&network, &manifest.assertions, &demo);
    let passed = checks.iter().all(|check| check.passed);

    Ok(LabRunReport {
        lab_id: manifest.id.clone(),
        passed,
        steps,
        checks,
        demo,
    })
}

pub fn setup_lab(manifest: &LabManifest, state_path: &str) -> Result<(), String> {
    let network = setup_network(&manifest.setup)?;
    save_network(&network, state_path)
}

pub fn verify_lab(
    manifest: &LabManifest,
    state_path: &str,
    demo: DemoFlags,
) -> Result<LabVerifyReport, String> {
    let network = load_network(state_path)?;
    let checks = evaluate_assertions(&network, &manifest.assertions, &demo);
    let passed = checks.iter().all(|check| check.passed);
    Ok(LabVerifyReport {
        lab_id: manifest.id.clone(),
        passed,
        checks,
        demo,
    })
}

fn apply_step(
    network: &mut BlockchainNetwork,
    step: &LabStep,
    demo: &mut DemoFlags,
) -> Result<StepResult, String> {
    match step {
        LabStep::CreateTransaction {
            from,
            to,
            amount_coin,
            fee_satoshi,
        } => {
            let amount = coin_to_satoshi(*amount_coin)?;
            let fee = fee_satoshi.unwrap_or(1_000);
            let recipient = network.get_node_address(*to);
            let created = network.create_transaction_with_fee(*from, &recipient, amount, fee);
            Ok(StepResult {
                action: "create_transaction".to_string(),
                ok: created.is_some(),
                detail: if created.is_some() {
                    format!(
                        "tx created from={} to={} amount_coin={} fee_satoshi={}",
                        from, to, amount_coin, fee
                    )
                } else {
                    format!(
                        "tx rejected from={} to={} amount_coin={} fee_satoshi={}",
                        from, to, amount_coin, fee
                    )
                },
            })
        }
        LabStep::MineBlock => {
            if network.current_val_id().is_none() {
                network.select_random_validator();
            }
            let mined = network.mine_block();
            Ok(StepResult {
                action: "mine_block".to_string(),
                ok: mined.is_some(),
                detail: if let Some(block) = mined {
                    format!("mined block index={}", block.index)
                } else {
                    "block mining failed".to_string()
                },
            })
        }
        LabStep::SetDifficulty { difficulty } => {
            network.set_difficulty(*difficulty);
            Ok(StepResult {
                action: "set_difficulty".to_string(),
                ok: true,
                detail: format!("difficulty set to {}", difficulty),
            })
        }
        LabStep::SimulateForkReorg { primary, secondary } => {
            let depth = network
                .simulate_fork_and_reorg(*primary, *secondary)
                .map_err(|err| format!("fork-reorg failed: {}", err))?;
            demo.reorg_depth = Some(depth);
            Ok(StepResult {
                action: "simulate_fork_reorg".to_string(),
                ok: true,
                detail: format!("reorg_depth={}", depth),
            })
        }
        LabStep::DemonstrateSignatureTamper {
            from,
            to,
            amount_coin,
        } => {
            let amount = coin_to_satoshi(*amount_coin)?;
            let recipient = network.get_node_address(*to);
            let Some(tx) = network.nodes.get(*from).and_then(|node| {
                node.wallet
                    .create_transaction_with_fee(&recipient, amount, 1_000)
            }) else {
                return Ok(StepResult {
                    action: "demonstrate_signature_tamper".to_string(),
                    ok: false,
                    detail: "could not create base transaction".to_string(),
                });
            };

            let mut tampered = tx.clone();
            if tampered.outputs.is_empty() {
                return Ok(StepResult {
                    action: "demonstrate_signature_tamper".to_string(),
                    ok: false,
                    detail: "transaction has no outputs".to_string(),
                });
            }
            tampered.outputs[0].amount = tampered.outputs[0].amount.saturating_add(1);
            tampered.id = tampered.calculate_hash();

            let rejected = !network.nodes[*from].verify_transaction(&tampered);
            demo.signature_tamper_rejected = rejected;
            Ok(StepResult {
                action: "demonstrate_signature_tamper".to_string(),
                ok: rejected,
                detail: if rejected {
                    "tampered transaction correctly rejected".to_string()
                } else {
                    "tampered transaction was unexpectedly accepted".to_string()
                },
            })
        }
        LabStep::DemonstrateDoubleSpend {
            from,
            to_a,
            to_b,
            amount_coin,
        } => {
            let amount = coin_to_satoshi(*amount_coin)?;
            let recipient_a = network.get_node_address(*to_a);
            let first = network.create_transaction_with_fee(*from, &recipient_a, amount, 1_000);
            if first.is_none() {
                return Ok(StepResult {
                    action: "demonstrate_double_spend".to_string(),
                    ok: false,
                    detail: "first spend could not be created".to_string(),
                });
            }

            let recipient_b = network.get_node_address(*to_b);
            let second = network.create_transaction_with_fee(*from, &recipient_b, amount, 1_000);
            let second_rejected = second.is_none();
            demo.double_spend_second_rejected = second_rejected;
            Ok(StepResult {
                action: "demonstrate_double_spend".to_string(),
                ok: second_rejected,
                detail: if second_rejected {
                    "second conflicting spend correctly rejected by mempool policy".to_string()
                } else {
                    "second conflicting spend was unexpectedly accepted".to_string()
                },
            })
        }
        LabStep::DemonstrateChainTamper { node_id } => {
            if network
                .nodes
                .get(*node_id)
                .map(|n| n.blockchain.len())
                .unwrap_or(0)
                < 2
            {
                if network.current_val_id().is_none() {
                    network.select_validator(0)?;
                }
                let _ = network.mine_block();
            }

            let Some(node) = network.nodes.get(*node_id) else {
                return Ok(StepResult {
                    action: "demonstrate_chain_tamper".to_string(),
                    ok: false,
                    detail: format!("node {} not found", node_id),
                });
            };
            let Some(last_block) = node.blockchain.last() else {
                return Ok(StepResult {
                    action: "demonstrate_chain_tamper".to_string(),
                    ok: false,
                    detail: "blockchain is empty".to_string(),
                });
            };

            let mut tampered_chain = node.blockchain.clone();
            let mut tampered_block = last_block.clone();
            tampered_block.hash = "deadbeef".to_string();
            if let Some(slot) = tampered_chain.last_mut() {
                *slot = tampered_block;
            }

            let rejected =
                !node.is_chain_valid_with_difficulty(&tampered_chain, network.difficulty);
            demo.chain_tamper_invalid = rejected;
            Ok(StepResult {
                action: "demonstrate_chain_tamper".to_string(),
                ok: rejected,
                detail: if rejected {
                    "tampered chain correctly rejected by difficulty-aware validation".to_string()
                } else {
                    "tampered chain was unexpectedly accepted".to_string()
                },
            })
        }
    }
}

fn evaluate_assertions(
    network: &BlockchainNetwork,
    assertions: &[LabAssertion],
    demo: &DemoFlags,
) -> Vec<AssertionResult> {
    assertions
        .iter()
        .map(|assertion| evaluate_assertion(network, assertion, demo))
        .collect()
}

fn evaluate_assertion(
    network: &BlockchainNetwork,
    assertion: &LabAssertion,
    demo: &DemoFlags,
) -> AssertionResult {
    match assertion {
        LabAssertion::NodeCount { expected } => {
            let actual = network.node_count();
            AssertionResult {
                assertion: format!("node_count=={}", expected),
                passed: actual == *expected,
                detail: format!("actual={}", actual),
            }
        }
        LabAssertion::BalanceEquals {
            node,
            amount_satoshi,
        } => {
            let actual = network
                .nodes
                .get(*node)
                .map(|n| n.wallet.get_balance())
                .unwrap_or(0);
            AssertionResult {
                assertion: format!("balance_equals node={} amount={}", node, amount_satoshi),
                passed: actual == *amount_satoshi,
                detail: format!("actual={}", actual),
            }
        }
        LabAssertion::BalanceMin {
            node,
            amount_satoshi,
        } => {
            let actual = network
                .nodes
                .get(*node)
                .map(|n| n.wallet.get_balance())
                .unwrap_or(0);
            AssertionResult {
                assertion: format!("balance_min node={} amount={}", node, amount_satoshi),
                passed: actual >= *amount_satoshi,
                detail: format!("actual={}", actual),
            }
        }
        LabAssertion::ChainHeightMin { expected } => {
            let actual = network
                .nodes
                .first()
                .map(|n| n.blockchain.len())
                .unwrap_or(0);
            AssertionResult {
                assertion: format!("chain_height_min>={}", expected),
                passed: actual >= *expected,
                detail: format!("actual={}", actual),
            }
        }
        LabAssertion::MempoolCountMin { expected } => {
            let actual = network.mempool.len();
            AssertionResult {
                assertion: format!("mempool_count_min>={}", expected),
                passed: actual >= *expected,
                detail: format!("actual={}", actual),
            }
        }
        LabAssertion::MempoolCountEquals { expected } => {
            let actual = network.mempool.len();
            AssertionResult {
                assertion: format!("mempool_count=={}", expected),
                passed: actual == *expected,
                detail: format!("actual={}", actual),
            }
        }
        LabAssertion::TipHashPresent => {
            let present = network
                .nodes
                .first()
                .and_then(|n| n.blockchain.last())
                .map(|b| !b.hash.is_empty())
                .unwrap_or(false);
            AssertionResult {
                assertion: "tip_hash_present".to_string(),
                passed: present,
                detail: format!("present={}", present),
            }
        }
        LabAssertion::DifficultyEquals { expected } => AssertionResult {
            assertion: format!("difficulty=={}", expected),
            passed: network.difficulty == *expected,
            detail: format!("actual={}", network.difficulty),
        },
        LabAssertion::CanonicalTipsMatch => {
            let passed = tips_match(network);
            AssertionResult {
                assertion: "canonical_tips_match".to_string(),
                passed,
                detail: if passed {
                    "all nodes share the same tip hash".to_string()
                } else {
                    "node tip hashes diverge".to_string()
                },
            }
        }
        LabAssertion::ReorgDepthEquals { expected } => {
            let actual = demo.reorg_depth;
            AssertionResult {
                assertion: format!("reorg_depth=={}", expected),
                passed: actual == Some(*expected),
                detail: format!("actual={:?}", actual),
            }
        }
        LabAssertion::DemoFlagTrue { key } => {
            let passed = match key.as_str() {
                "signature_tamper_rejected" => demo.signature_tamper_rejected,
                "double_spend_second_rejected" => demo.double_spend_second_rejected,
                "chain_tamper_invalid" => demo.chain_tamper_invalid,
                _ => false,
            };
            AssertionResult {
                assertion: format!("demo_flag_true:{}", key),
                passed,
                detail: format!("value={}", passed),
            }
        }
    }
}

fn tips_match(network: &BlockchainNetwork) -> bool {
    let tips: Vec<Option<&str>> = network
        .nodes
        .iter()
        .map(|node| node.blockchain.last().map(|block| block.hash.as_str()))
        .collect();
    if tips.is_empty() {
        return false;
    }
    tips.iter().all(|tip| tip == &tips[0]) && tips[0].is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("lab_{}_{}_{}", name, std::process::id(), stamp))
    }

    #[test]
    fn utxo_lab_run_should_pass_with_bundled_manifest() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("labs");
        let (manifest, _) = load_lab(&root, "01-utxo-and-transfers").expect("lab should load");
        let dir = unique_temp_dir("utxo");
        fs::create_dir_all(&dir).expect("dir");
        let state_path = dir.join("state.json");
        let report = run_lab(&manifest, state_path.to_string_lossy().as_ref()).expect("run");
        assert!(report.passed, "{:?}", report);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn fork_lab_run_should_pass_with_bundled_manifest() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("labs");
        let (manifest, _) = load_lab(&root, "05-forks-and-reorgs").expect("lab should load");
        let dir = unique_temp_dir("fork");
        fs::create_dir_all(&dir).expect("dir");
        let state_path = dir.join("state.json");
        let report = run_lab(&manifest, state_path.to_string_lossy().as_ref()).expect("run");
        assert!(report.passed, "{:?}", report);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn discover_labs_should_find_all_modules() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("labs");
        let labs = discover_labs(&root).expect("discover");
        assert!(
            labs.len() >= 6,
            "expected at least 6 labs, got {}",
            labs.len()
        );
    }

    #[test]
    fn all_bundled_labs_should_pass_run() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("labs");
        let labs = discover_labs(&root).expect("discover");
        assert_eq!(labs.len(), 6);

        for summary in labs {
            let (manifest, _) = load_lab(&root, &summary.id).expect("load");
            let dir = unique_temp_dir(&summary.id);
            fs::create_dir_all(&dir).expect("dir");
            let state_path = dir.join("state.json");
            let report =
                run_lab(&manifest, state_path.to_string_lossy().as_ref()).expect("run lab");
            assert!(report.passed, "lab {} failed: {:?}", summary.id, report);
            let _ = fs::remove_dir_all(dir);
        }
    }
}
