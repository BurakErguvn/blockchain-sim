use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::lab::{
    coin_to_satoshi, load_network, save_network, setup_lab, DemoFlags, GuidedCheck, GuidedStep,
    LabAssertion, LabManifest,
};
use crate::network::BlockchainNetwork;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningStatus {
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedStep {
    pub step_id: String,
    pub completed_at: u64,
    pub attempts: u32,
    pub hints_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptRecord {
    pub step_id: String,
    pub at: u64,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSession {
    pub session_id: String,
    pub lab_id: String,
    pub state_path: String,
    pub current_step: usize,
    pub completed_steps: Vec<CompletedStep>,
    pub attempts: Vec<AttemptRecord>,
    pub hints_used_total: u32,
    pub hint_level_by_step: Vec<u32>,
    pub started_at: u64,
    pub updated_at: u64,
    pub status: LearningStatus,
    #[serde(default)]
    pub demo: DemoFlags,
    #[serde(default)]
    pub acknowledgements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActiveLearningPointer {
    session_id: String,
    lab_id: String,
    session_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuidedStepView {
    pub index: usize,
    pub total: usize,
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub concept: String,
    pub command_template: String,
    pub discussion_prompt: String,
    pub hints_available: usize,
    pub hints_revealed: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnStartReport {
    pub session_id: String,
    pub lab_id: String,
    pub title: String,
    pub aliases: Vec<String>,
    pub state_path: String,
    pub step: GuidedStepView,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnStatusReport {
    pub session_id: String,
    pub lab_id: String,
    pub status: LearningStatus,
    pub current_step: usize,
    pub total_steps: usize,
    pub completed_count: usize,
    pub attempts: usize,
    pub hints_used_total: u32,
    pub progress_percent: u32,
    pub step: Option<GuidedStepView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnHintReport {
    pub step_id: String,
    pub hint_level: u32,
    pub hint: String,
    pub more_hints: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnCheckReport {
    pub step_id: String,
    pub passed: bool,
    pub observed: String,
    pub likely_cause: String,
    pub next_action: String,
    pub completed: bool,
    pub lab_completed: bool,
    pub step: Option<GuidedStepView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnNextReport {
    pub advanced: bool,
    pub lab_completed: bool,
    pub step: Option<GuidedStepView>,
    pub message: String,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn make_session_id(lab_id: &str) -> String {
    format!("learn-{}-{}", lab_id, now_secs())
}

pub fn sessions_dir_for(state_path: &str) -> PathBuf {
    let parent = Path::new(state_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    parent.join("learning_sessions")
}

fn active_pointer_path(state_path: &str) -> PathBuf {
    let parent = Path::new(state_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    parent.join(".sim_cli_learning_active.json")
}

fn session_file_path(state_path: &str, session_id: &str) -> PathBuf {
    sessions_dir_for(state_path).join(format!("{}.json", session_id))
}

pub fn guided_steps_of(manifest: &LabManifest) -> &[GuidedStep] {
    &manifest.guided_steps
}

pub fn require_guided_steps(manifest: &LabManifest) -> Result<&[GuidedStep], String> {
    if manifest.guided_steps.is_empty() {
        return Err(format!(
            "Lab '{}' henüz yönlendirmeli adım içermiyor (guided_steps)",
            manifest.id
        ));
    }
    Ok(&manifest.guided_steps)
}

pub fn save_session(session: &LearningSession) -> Result<(), String> {
    let path = session_file_path(&session.state_path, &session.session_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let content = serde_json::to_vec_pretty(session).map_err(|err| err.to_string())?;
    fs::write(&path, content).map_err(|err| err.to_string())?;

    let pointer = ActiveLearningPointer {
        session_id: session.session_id.clone(),
        lab_id: session.lab_id.clone(),
        session_path: path.display().to_string(),
    };
    let pointer_path = active_pointer_path(&session.state_path);
    let pointer_json = serde_json::to_vec_pretty(&pointer).map_err(|err| err.to_string())?;
    fs::write(pointer_path, pointer_json).map_err(|err| err.to_string())?;
    Ok(())
}

pub fn load_session_from_path(path: &Path) -> Result<LearningSession, String> {
    let content = fs::read(path).map_err(|err| format!("Session okunamadı: {}", err))?;
    serde_json::from_slice(&content).map_err(|err| format!("Session parse hatası: {}", err))
}

pub fn load_active_session(state_path: &str) -> Result<LearningSession, String> {
    let pointer_path = active_pointer_path(state_path);
    if !pointer_path.exists() {
        return Err("Aktif öğrenme oturumu yok. `learn start <lab-id>` ile başlatın.".to_string());
    }
    let content = fs::read(&pointer_path).map_err(|err| err.to_string())?;
    let pointer: ActiveLearningPointer =
        serde_json::from_slice(&content).map_err(|err| err.to_string())?;
    let session_path = PathBuf::from(&pointer.session_path);
    let session = load_session_from_path(&session_path)?;
    if session.state_path != state_path {
        return Err(format!(
            "Aktif oturum farklı state_path kullanıyor ({}). Aynı --state-path / profil ile devam edin.",
            session.state_path
        ));
    }
    Ok(session)
}

pub fn clear_active_session(state_path: &str) -> Result<(), String> {
    let pointer_path = active_pointer_path(state_path);
    if pointer_path.exists() {
        fs::remove_file(&pointer_path).map_err(|err| err.to_string())?;
    }
    Ok(())
}

pub fn step_view(manifest: &LabManifest, session: &LearningSession) -> Option<GuidedStepView> {
    let steps = guided_steps_of(manifest);
    let index = session.current_step;
    let step = steps.get(index)?;
    let hints_revealed = session.hint_level_by_step.get(index).copied().unwrap_or(0);
    Some(GuidedStepView {
        index: index + 1,
        total: steps.len(),
        id: step.id.clone(),
        title: step.title.clone(),
        instruction: step.instruction.clone(),
        concept: step.concept.clone(),
        command_template: step.command_template.clone(),
        discussion_prompt: step.discussion_prompt.clone(),
        hints_available: step.hint_levels.len(),
        hints_revealed,
    })
}

pub fn start_learning(
    manifest: &LabManifest,
    state_path: &str,
) -> Result<(LearningSession, LearnStartReport), String> {
    let steps = require_guided_steps(manifest)?;
    setup_lab(manifest, state_path)?;

    let now = now_secs();
    let session = LearningSession {
        session_id: make_session_id(&manifest.id),
        lab_id: manifest.id.clone(),
        state_path: state_path.to_string(),
        current_step: 0,
        completed_steps: Vec::new(),
        attempts: Vec::new(),
        hints_used_total: 0,
        hint_level_by_step: vec![0; steps.len()],
        started_at: now,
        updated_at: now,
        status: LearningStatus::InProgress,
        demo: DemoFlags::default(),
        acknowledgements: Vec::new(),
    };
    save_session(&session)?;

    let step = step_view(manifest, &session).ok_or("İlk adım bulunamadı")?;
    let report = LearnStartReport {
        session_id: session.session_id.clone(),
        lab_id: session.lab_id.clone(),
        title: manifest.title.clone(),
        aliases: manifest.aliases.clone(),
        state_path: state_path.to_string(),
        step,
    };
    Ok((session, report))
}

pub fn status_report(
    manifest: &LabManifest,
    session: &LearningSession,
) -> Result<LearnStatusReport, String> {
    let total = require_guided_steps(manifest)?.len();
    let completed = session.completed_steps.len();
    let progress = if total == 0 {
        0
    } else {
        ((completed * 100) / total) as u32
    };
    Ok(LearnStatusReport {
        session_id: session.session_id.clone(),
        lab_id: session.lab_id.clone(),
        status: session.status.clone(),
        current_step: session.current_step.min(total.saturating_sub(1)),
        total_steps: total,
        completed_count: completed,
        attempts: session.attempts.len(),
        hints_used_total: session.hints_used_total,
        progress_percent: progress,
        step: if session.status == LearningStatus::Completed {
            None
        } else {
            step_view(manifest, session)
        },
    })
}

pub fn reveal_hint(
    manifest: &LabManifest,
    session: &mut LearningSession,
) -> Result<LearnHintReport, String> {
    let steps = require_guided_steps(manifest)?;
    if session.status == LearningStatus::Completed {
        return Err("Laboratuvar zaten tamamlandı".to_string());
    }
    let index = session.current_step;
    let step = steps
        .get(index)
        .ok_or_else(|| "Geçerli adım bulunamadı".to_string())?;
    if step.hint_levels.is_empty() {
        return Err("Bu adım için ipucu tanımlı değil".to_string());
    }

    if session.hint_level_by_step.len() < steps.len() {
        session.hint_level_by_step.resize(steps.len(), 0);
    }

    let revealed = session.hint_level_by_step[index];
    let next_level = revealed.min(step.hint_levels.len() as u32);
    if next_level as usize >= step.hint_levels.len() {
        let hint = step.hint_levels.last().cloned().unwrap_or_default();
        return Ok(LearnHintReport {
            step_id: step.id.clone(),
            hint_level: revealed,
            hint,
            more_hints: false,
        });
    }

    let hint = step.hint_levels[next_level as usize].clone();
    session.hint_level_by_step[index] = next_level + 1;
    session.hints_used_total = session.hints_used_total.saturating_add(1);
    session.updated_at = now_secs();
    save_session(session)?;

    Ok(LearnHintReport {
        step_id: step.id.clone(),
        hint_level: next_level + 1,
        hint,
        more_hints: (next_level as usize + 1) < step.hint_levels.len(),
    })
}

pub fn check_current_step(
    manifest: &LabManifest,
    session: &mut LearningSession,
    ack_token: Option<&str>,
) -> Result<LearnCheckReport, String> {
    let steps = require_guided_steps(manifest)?;
    if session.status == LearningStatus::Completed {
        return Ok(LearnCheckReport {
            step_id: "completed".to_string(),
            passed: true,
            observed: "Laboratuvar tamamlanmış".to_string(),
            likely_cause: String::new(),
            next_action: "`learn status` ile özeti görüntüleyin".to_string(),
            completed: true,
            lab_completed: true,
            step: None,
        });
    }

    let index = session.current_step;
    let step = steps
        .get(index)
        .ok_or_else(|| "Geçerli adım bulunamadı".to_string())?
        .clone();

    let mut network = load_network(&session.state_path)?;
    let evaluation = evaluate_guided_check(
        &mut network,
        &step.check,
        &mut session.demo,
        ack_token,
        &mut session.acknowledgements,
    )?;
    // Persist network if a demo mutated it.
    save_network(&network, &session.state_path)?;

    let hints_used = session.hint_level_by_step.get(index).copied().unwrap_or(0);
    let attempt_count = session
        .attempts
        .iter()
        .filter(|a| a.step_id == step.id)
        .count() as u32
        + 1;

    session.attempts.push(AttemptRecord {
        step_id: step.id.clone(),
        at: now_secs(),
        passed: evaluation.passed,
        detail: evaluation.observed.clone(),
    });
    session.updated_at = now_secs();

    let mut lab_completed = false;
    let mut completed = false;
    let mut next_step = None;

    if evaluation.passed {
        completed = true;
        if !session
            .completed_steps
            .iter()
            .any(|item| item.step_id == step.id)
        {
            session.completed_steps.push(CompletedStep {
                step_id: step.id.clone(),
                completed_at: now_secs(),
                attempts: attempt_count,
                hints_used,
            });
        }

        if index + 1 >= steps.len() {
            session.status = LearningStatus::Completed;
            lab_completed = true;
        } else {
            session.current_step = index + 1;
            next_step = step_view(manifest, session);
        }
    }

    save_session(session)?;

    let feedback = if evaluation.passed {
        PedagogicalFeedback {
            observed: evaluation.observed,
            likely_cause: "Beklenen durum doğrulandı.".to_string(),
            next_action: if lab_completed {
                "Laboratuvar tamamlandı. `learn status` ile özeti inceleyin.".to_string()
            } else {
                "Sonraki göreve geçildi. `learn status` ile yeni adımı görün.".to_string()
            },
        }
    } else {
        pedagogical_feedback(&step, &evaluation.observed)
    };

    Ok(LearnCheckReport {
        step_id: step.id,
        passed: evaluation.passed,
        observed: feedback.observed,
        likely_cause: feedback.likely_cause,
        next_action: feedback.next_action,
        completed,
        lab_completed,
        step: next_step,
    })
}

pub fn advance_step(
    manifest: &LabManifest,
    session: &mut LearningSession,
    force: bool,
) -> Result<LearnNextReport, String> {
    let steps = require_guided_steps(manifest)?;
    if session.status == LearningStatus::Completed {
        return Ok(LearnNextReport {
            advanced: false,
            lab_completed: true,
            step: None,
            message: "Laboratuvar zaten tamamlandı".to_string(),
        });
    }

    let index = session.current_step;
    let current = steps
        .get(index)
        .ok_or_else(|| "Geçerli adım bulunamadı".to_string())?;
    let current_done = session
        .completed_steps
        .iter()
        .any(|item| item.step_id == current.id);

    if !current_done && !force {
        return Ok(LearnNextReport {
            advanced: false,
            lab_completed: false,
            step: step_view(manifest, session),
            message: "Önce mevcut adımı `learn check` ile tamamlayın (öğretmen için --force)"
                .to_string(),
        });
    }

    if index + 1 >= steps.len() {
        session.status = LearningStatus::Completed;
        session.updated_at = now_secs();
        save_session(session)?;
        return Ok(LearnNextReport {
            advanced: false,
            lab_completed: true,
            step: None,
            message: "Tüm adımlar tamamlandı".to_string(),
        });
    }

    session.current_step = index + 1;
    session.updated_at = now_secs();
    save_session(session)?;
    Ok(LearnNextReport {
        advanced: true,
        lab_completed: false,
        step: step_view(manifest, session),
        message: "Sonraki adıma geçildi".to_string(),
    })
}

pub fn reset_learning(state_path: &str, lab_id: Option<&str>) -> Result<String, String> {
    if let Ok(session) = load_active_session(state_path) {
        if lab_id.is_none() || lab_id == Some(session.lab_id.as_str()) {
            let path = session_file_path(state_path, &session.session_id);
            if path.exists() {
                fs::remove_file(&path).map_err(|err| err.to_string())?;
            }
        }
    }
    clear_active_session(state_path)?;
    if Path::new(state_path).exists() {
        fs::remove_file(state_path).map_err(|err| err.to_string())?;
    }
    Ok(match lab_id {
        Some(id) => format!("Öğrenme oturumu ve state sıfırlandı ({})", id),
        None => "Aktif öğrenme oturumu ve state sıfırlandı".to_string(),
    })
}

struct CheckEvaluation {
    passed: bool,
    observed: String,
}

struct PedagogicalFeedback {
    observed: String,
    likely_cause: String,
    next_action: String,
}

fn evaluate_guided_check(
    network: &mut BlockchainNetwork,
    check: &GuidedCheck,
    demo: &mut DemoFlags,
    ack_token: Option<&str>,
    acknowledgements: &mut Vec<String>,
) -> Result<CheckEvaluation, String> {
    match check {
        GuidedCheck::NodeCount { expected } => Ok(from_assertion(
            network,
            &LabAssertion::NodeCount {
                expected: *expected,
            },
            demo,
        )),
        GuidedCheck::BalanceMin {
            node,
            amount_satoshi,
        } => Ok(from_assertion(
            network,
            &LabAssertion::BalanceMin {
                node: *node,
                amount_satoshi: *amount_satoshi,
            },
            demo,
        )),
        GuidedCheck::BalanceEquals {
            node,
            amount_satoshi,
        } => Ok(from_assertion(
            network,
            &LabAssertion::BalanceEquals {
                node: *node,
                amount_satoshi: *amount_satoshi,
            },
            demo,
        )),
        GuidedCheck::MempoolCountMin { expected } => Ok(from_assertion(
            network,
            &LabAssertion::MempoolCountMin {
                expected: *expected,
            },
            demo,
        )),
        GuidedCheck::MempoolCountEquals { expected } => Ok(from_assertion(
            network,
            &LabAssertion::MempoolCountEquals {
                expected: *expected,
            },
            demo,
        )),
        GuidedCheck::ChainHeightMin { expected } => Ok(from_assertion(
            network,
            &LabAssertion::ChainHeightMin {
                expected: *expected,
            },
            demo,
        )),
        GuidedCheck::TipHashPresent => {
            Ok(from_assertion(network, &LabAssertion::TipHashPresent, demo))
        }
        GuidedCheck::CanonicalTipsMatch => Ok(from_assertion(
            network,
            &LabAssertion::CanonicalTipsMatch,
            demo,
        )),
        GuidedCheck::DifficultyEquals { expected } => Ok(from_assertion(
            network,
            &LabAssertion::DifficultyEquals {
                expected: *expected,
            },
            demo,
        )),
        GuidedCheck::RunSignatureTamperDemo {
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
                return Ok(CheckEvaluation {
                    passed: false,
                    observed:
                        "İmza demosu için temel işlem oluşturulamadı (fonlanmış node gerekli)"
                            .to_string(),
                });
            };
            let mut tampered = tx;
            if tampered.outputs.is_empty() {
                return Ok(CheckEvaluation {
                    passed: false,
                    observed: "İşlemde çıktı yok".to_string(),
                });
            }
            tampered.outputs[0].amount = tampered.outputs[0].amount.saturating_add(1);
            tampered.id = tampered.calculate_hash();
            let rejected = !network.nodes[*from].verify_transaction(&tampered);
            demo.signature_tamper_rejected = rejected;
            Ok(CheckEvaluation {
                passed: rejected,
                observed: if rejected {
                    "Değiştirilmiş işlem imza doğrulamasında reddedildi".to_string()
                } else {
                    "Değiştirilmiş işlem beklenmedik biçimde kabul edildi".to_string()
                },
            })
        }
        GuidedCheck::RunForkReorgDemo {
            primary,
            secondary,
            expected_depth,
        } => {
            if network.current_val_id().is_none() {
                let _ = network.select_validator(0);
            }
            if network
                .nodes
                .first()
                .map(|n| n.blockchain.len())
                .unwrap_or(0)
                < 2
            {
                network
                    .mine_block()
                    .ok_or_else(|| "Fork demosu için ek blok üretilemedi".to_string())?;
            }
            let depth = network
                .simulate_fork_and_reorg(*primary, *secondary)
                .map_err(|err| format!("Fork/reorg demosu başarısız: {}", err))?;
            demo.reorg_depth = Some(depth);
            let tips_ok = lab_tips_match(network);
            let passed = depth == *expected_depth && tips_ok;
            Ok(CheckEvaluation {
                passed,
                observed: format!("reorg_depth={} tips_consistent={}", depth, tips_ok),
            })
        }
        GuidedCheck::Acknowledge { token } => {
            let expected = if token.is_empty() {
                "understood"
            } else {
                token.as_str()
            };
            let provided = ack_token.unwrap_or("").trim();
            let passed = provided == expected;
            if passed && !acknowledgements.iter().any(|item| item == expected) {
                acknowledgements.push(expected.to_string());
            }
            Ok(CheckEvaluation {
                passed,
                observed: if passed {
                    format!("Kavram onayı alındı ({})", expected)
                } else {
                    format!("Onay bekleniyor. `learn check --ack {}` kullanın", expected)
                },
            })
        }
    }
}

fn from_assertion(
    network: &BlockchainNetwork,
    assertion: &LabAssertion,
    demo: &DemoFlags,
) -> CheckEvaluation {
    // Reuse lab assertion evaluator via a tiny local reimplementation for state checks.
    let result = match assertion {
        LabAssertion::NodeCount { expected } => {
            let actual = network.node_count();
            (actual == *expected, format!("node_count actual={}", actual))
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
            (
                actual >= *amount_satoshi,
                format!("balance node={} actual={}", node, actual),
            )
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
            (
                actual == *amount_satoshi,
                format!("balance node={} actual={}", node, actual),
            )
        }
        LabAssertion::MempoolCountMin { expected } => {
            let actual = network.mempool.len();
            (
                actual >= *expected,
                format!("mempool_count actual={}", actual),
            )
        }
        LabAssertion::MempoolCountEquals { expected } => {
            let actual = network.mempool.len();
            (
                actual == *expected,
                format!("mempool_count actual={}", actual),
            )
        }
        LabAssertion::ChainHeightMin { expected } => {
            let actual = network
                .nodes
                .first()
                .map(|n| n.blockchain.len())
                .unwrap_or(0);
            (
                actual >= *expected,
                format!("chain_height actual={}", actual),
            )
        }
        LabAssertion::TipHashPresent => {
            let present = network
                .nodes
                .first()
                .and_then(|n| n.blockchain.last())
                .map(|b| !b.hash.is_empty())
                .unwrap_or(false);
            (present, format!("tip_hash_present={}", present))
        }
        LabAssertion::DifficultyEquals { expected } => (
            network.difficulty == *expected,
            format!("difficulty actual={}", network.difficulty),
        ),
        LabAssertion::CanonicalTipsMatch => {
            let passed = lab_tips_match(network);
            (passed, format!("canonical_tips_match={}", passed))
        }
        _ => (false, "unsupported assertion in guided mode".to_string()),
    };
    let _ = demo;
    CheckEvaluation {
        passed: result.0,
        observed: result.1,
    }
}

fn lab_tips_match(network: &BlockchainNetwork) -> bool {
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

fn pedagogical_feedback(step: &GuidedStep, observed: &str) -> PedagogicalFeedback {
    let observed_l = observed.to_lowercase();
    let (likely_cause, next_action) = if observed_l.contains("mempool") {
        (
            "Beklenen işlem henüz mempool'da görünmüyor veya sayısı yetersiz.".to_string(),
            "Alice'ten Bob'a doğru miktarla `tx create` deneyin; hata mesajını okuyun.".to_string(),
        )
    } else if observed_l.contains("balance") {
        (
            "Bakiye henüz beklenen değere ulaşmamış olabilir; onaylanmamış işlem bakiyeyi değiştirmez."
                .to_string(),
            "`nodes list` ile bakiyeleri kontrol edin; gerekirse `mine once` çalıştırın.".to_string(),
        )
    } else if observed_l.contains("chain_height") {
        (
            "Zincir yüksekliği beklenen eşiğin altında.".to_string(),
            "`mine once` ile yeni blok üretip `chain tip` ile doğrulayın.".to_string(),
        )
    } else if observed_l.contains("imza") || observed_l.contains("signature") {
        (
            "İmza demosu için fonlanmış bir gönderen veya geçerli işlem gerekli.".to_string(),
            "Önce geçerli bir transfer oluşturun, sonra bu adımı yeniden kontrol edin.".to_string(),
        )
    } else if observed_l.contains("onay") || observed_l.contains("ack") {
        (
            "Kavram onayı henüz verilmedi.".to_string(),
            step.feedback_fail.clone().if_empty_then(|| {
                "Tartışma metnini okuyup `learn check --ack understood` çalıştırın.".to_string()
            }),
        )
    } else if !step.feedback_fail.trim().is_empty() {
        (
            step.feedback_fail.clone(),
            "Komut şablonunu ve ipuçlarını gözden geçirin (`learn hint`).".to_string(),
        )
    } else {
        (
            "Beklenen ağ durumu henüz oluşmamış.".to_string(),
            "Görev yönergesini tekrar okuyup `learn hint` ile ilerleyin.".to_string(),
        )
    };

    PedagogicalFeedback {
        observed: observed.to_string(),
        likely_cause,
        next_action,
    }
}

trait IfEmpty {
    fn if_empty_then(self, f: impl FnOnce() -> String) -> String;
}

impl IfEmpty for String {
    fn if_empty_then(self, f: impl FnOnce() -> String) -> String {
        if self.trim().is_empty() {
            f()
        } else {
            self
        }
    }
}

/// Used by tests and CLI helpers to ensure guided manifests parse.
pub fn assert_guided_ready(manifest: &LabManifest) -> Result<(), String> {
    require_guided_steps(manifest).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lab::load_lab;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("learn_{}_{}_{}", name, std::process::id(), stamp))
    }

    #[test]
    fn guided_utxo_flow_can_be_completed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("labs");
        let (manifest, _) = load_lab(&root, "01-utxo-and-transfers").expect("load");
        assert!(require_guided_steps(&manifest).is_ok());

        let dir = unique_temp_dir("utxo");
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();

        let (mut session, _start) = start_learning(&manifest, &state).expect("start");

        // Step 0: inspect funded alice
        let report = check_current_step(&manifest, &mut session, None).expect("check0");
        assert!(report.passed, "{:?}", report);

        // Step 1: create transfer
        let mut network = load_network(&state).unwrap();
        let recipient = network.get_node_address(1);
        assert!(network
            .create_transaction_with_fee(0, &recipient, 1_000_000_000, 1_000)
            .is_some());
        save_network(&network, &state).unwrap();
        let report = check_current_step(&manifest, &mut session, None).expect("check1");
        assert!(report.passed, "{:?}", report);

        // Step 2: mempool observed (already true) / or mine
        // Continue until complete by performing required actions based on remaining steps.
        while session.status != LearningStatus::Completed {
            let step = &manifest.guided_steps[session.current_step];
            match &step.check {
                GuidedCheck::MempoolCountMin { .. } | GuidedCheck::MempoolCountEquals { .. } => {
                    let report = check_current_step(&manifest, &mut session, None).unwrap();
                    if !report.passed {
                        let mut network = load_network(&state).unwrap();
                        if network.mempool.is_empty() {
                            let recipient = network.get_node_address(1);
                            let _ = network.create_transaction_with_fee(
                                0,
                                &recipient,
                                100_000_000,
                                1_000,
                            );
                            save_network(&network, &state).unwrap();
                        }
                        let report = check_current_step(&manifest, &mut session, None).unwrap();
                        assert!(report.passed, "{:?}", report);
                    }
                }
                GuidedCheck::ChainHeightMin { .. } | GuidedCheck::TipHashPresent => {
                    let mut network = load_network(&state).unwrap();
                    if network.current_val_id().is_none() {
                        let _ = network.select_validator(0);
                    }
                    let _ = network.mine_block();
                    save_network(&network, &state).unwrap();
                    let report = check_current_step(&manifest, &mut session, None).unwrap();
                    assert!(report.passed, "{:?}", report);
                }
                GuidedCheck::BalanceEquals {
                    amount_satoshi: 0, ..
                } => {
                    // Unconfirmed-balance observation: do not mine first.
                    let report = check_current_step(&manifest, &mut session, None).unwrap();
                    assert!(report.passed, "{:?}", report);
                }
                GuidedCheck::BalanceMin { .. } | GuidedCheck::BalanceEquals { .. } => {
                    let mut network = load_network(&state).unwrap();
                    if network.mempool.is_empty() {
                        let recipient = network.get_node_address(1);
                        let _ = network.create_transaction_with_fee(
                            0,
                            &recipient,
                            1_000_000_000,
                            1_000,
                        );
                    }
                    if network.current_val_id().is_none() {
                        let _ = network.select_validator(0);
                    }
                    let _ = network.mine_block();
                    save_network(&network, &state).unwrap();
                    let report = check_current_step(&manifest, &mut session, None).unwrap();
                    assert!(report.passed, "{:?}", report);
                }
                GuidedCheck::Acknowledge { token } => {
                    let ack = if token.is_empty() {
                        "understood"
                    } else {
                        token.as_str()
                    };
                    let report = check_current_step(&manifest, &mut session, Some(ack)).unwrap();
                    assert!(report.passed, "{:?}", report);
                }
                other => {
                    let report = check_current_step(&manifest, &mut session, None).unwrap();
                    assert!(
                        report.passed,
                        "unexpected failing check {:?}: {:?}",
                        other, report
                    );
                }
            }
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn hint_levels_increment() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("labs");
        let (manifest, _) = load_lab(&root, "01-utxo-and-transfers").expect("load");
        let dir = unique_temp_dir("hint");
        fs::create_dir_all(&dir).unwrap();
        let state = dir.join("state.json").to_string_lossy().to_string();
        let (mut session, _) = start_learning(&manifest, &state).unwrap();
        let h1 = reveal_hint(&manifest, &mut session).unwrap();
        let h2 = reveal_hint(&manifest, &mut session).unwrap();
        assert_eq!(h1.hint_level, 1);
        assert_eq!(h2.hint_level, 2);
        assert_ne!(h1.hint, h2.hint);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn signature_and_fork_guided_labs_start_and_progress() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("labs");
        for lab_id in ["02-signatures-and-integrity", "05-forks-and-reorgs"] {
            let (manifest, _) = load_lab(&root, lab_id).expect("load");
            assert!(require_guided_steps(&manifest).is_ok());
            let dir = unique_temp_dir(lab_id);
            fs::create_dir_all(&dir).unwrap();
            let state = dir.join("state.json").to_string_lossy().to_string();
            let (mut session, start) = start_learning(&manifest, &state).unwrap();
            assert_eq!(start.lab_id, lab_id);
            // First checks in both labs are pure state inspections.
            let report = check_current_step(&manifest, &mut session, None).unwrap();
            assert!(report.passed, "{} first check failed: {:?}", lab_id, report);
            let _ = fs::remove_dir_all(dir);
        }
    }
}
