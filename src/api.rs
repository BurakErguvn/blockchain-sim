use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::network::BlockchainNetwork;

#[derive(Clone)]
pub struct ApiState {
    pub network: Arc<Mutex<BlockchainNetwork>>,
    pub state_path: String,
}

#[derive(Serialize)]
struct ApiError {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
pub struct NetworkStateResponse {
    node_count: usize,
    current_validator_id: Option<usize>,
    difficulty: usize,
    block_time: u64,
    mempool_count: usize,
    tip_height: Option<usize>,
    tip_hash: Option<String>,
}

#[derive(Serialize)]
pub struct NodeSummary {
    id: usize,
    address: String,
    balance: u64,
    connections: usize,
    is_validator: bool,
    blockchain_len: usize,
}

#[derive(Serialize)]
pub struct NodeDetail {
    id: usize,
    address: String,
    balance: u64,
    connections: Vec<usize>,
    is_validator: bool,
    blockchain_len: usize,
    utxo_count: usize,
}

#[derive(Serialize)]
pub struct BlockSummary {
    index: usize,
    hash: String,
    previous_hash: String,
    timestamp: u64,
    transaction_count: usize,
}

#[derive(Serialize)]
pub struct MempoolEntry {
    id: String,
    sender: String,
    output_count: usize,
    total_output_amount: u64,
    estimated_size_bytes: usize,
    fee_rate_sat_per_kb: Option<u64>,
}

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    sender_id: usize,
    recipient_id: usize,
    amount_coin: f64,
}

#[derive(Serialize)]
pub struct CreateTransactionResponse {
    tx_id: String,
}

#[derive(Serialize)]
pub struct MineBlockResponse {
    index: usize,
    hash: String,
    transaction_count: usize,
}

pub fn router(network: Arc<Mutex<BlockchainNetwork>>, state_path: impl Into<String>) -> Router {
    let state = ApiState {
        network,
        state_path: state_path.into(),
    };

    Router::new()
        .route("/health", get(health))
        .route("/network/state", get(network_state))
        .route("/nodes", get(nodes))
        .route("/nodes/{id}", get(node_detail))
        .route("/nodes/{id}/blockchain", get(node_blockchain))
        .route("/mempool", get(mempool))
        .route("/transactions", post(create_transaction))
        .route("/mine", post(mine_block))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn network_state(
    State(state): State<ApiState>,
) -> Result<Json<NetworkStateResponse>, ApiHttpError> {
    let network = state
        .network
        .lock()
        .map_err(|_| ApiHttpError::internal("Ağ durumu alınamadı"))?;

    let tip = network
        .nodes
        .first()
        .and_then(|node| node.blockchain.last())
        .map(|block| (block.index, block.hash.clone()));

    Ok(Json(NetworkStateResponse {
        node_count: network.node_count(),
        current_validator_id: network.current_val_id(),
        difficulty: network.difficulty,
        block_time: network.block_time,
        mempool_count: network.mempool.len(),
        tip_height: tip.as_ref().map(|(height, _)| *height),
        tip_hash: tip.as_ref().map(|(_, hash)| hash.clone()),
    }))
}

async fn nodes(State(state): State<ApiState>) -> Result<Json<Vec<NodeSummary>>, ApiHttpError> {
    let network = state
        .network
        .lock()
        .map_err(|_| ApiHttpError::internal("Node listesi alınamadı"))?;

    let items = network
        .nodes
        .iter()
        .map(|node| NodeSummary {
            id: node.id,
            address: node.wallet.get_address().to_string(),
            balance: node.wallet.get_balance(),
            connections: node.connections.len(),
            is_validator: node.is_validator,
            blockchain_len: node.blockchain.len(),
        })
        .collect();

    Ok(Json(items))
}

async fn node_detail(
    Path(node_id): Path<usize>,
    State(state): State<ApiState>,
) -> Result<Json<NodeDetail>, ApiHttpError> {
    let network = state
        .network
        .lock()
        .map_err(|_| ApiHttpError::internal("Node detayı alınamadı"))?;

    let Some(node) = network.nodes.get(node_id) else {
        return Err(ApiHttpError::not_found("Node bulunamadı"));
    };

    Ok(Json(NodeDetail {
        id: node.id,
        address: node.wallet.get_address().to_string(),
        balance: node.wallet.get_balance(),
        connections: node.connections.clone(),
        is_validator: node.is_validator,
        blockchain_len: node.blockchain.len(),
        utxo_count: node.utxo_set.len(),
    }))
}

async fn node_blockchain(
    Path(node_id): Path<usize>,
    State(state): State<ApiState>,
) -> Result<Json<Vec<BlockSummary>>, ApiHttpError> {
    let network = state
        .network
        .lock()
        .map_err(|_| ApiHttpError::internal("Blockchain alınamadı"))?;

    let Some(node) = network.nodes.get(node_id) else {
        return Err(ApiHttpError::not_found("Node bulunamadı"));
    };

    let blocks = node
        .blockchain
        .iter()
        .map(|block| BlockSummary {
            index: block.index,
            hash: block.hash.clone(),
            previous_hash: block.previous_hash.clone(),
            timestamp: block.timestamp,
            transaction_count: block.transactions.len(),
        })
        .collect();

    Ok(Json(blocks))
}

async fn mempool(State(state): State<ApiState>) -> Result<Json<Vec<MempoolEntry>>, ApiHttpError> {
    let network = state
        .network
        .lock()
        .map_err(|_| ApiHttpError::internal("Mempool alınamadı"))?;

    let fee_reference_utxo = network.nodes.first().map(|node| &node.utxo_set);
    let entries = network
        .mempool
        .iter()
        .map(|tx| MempoolEntry {
            id: tx.id.clone(),
            sender: tx
                .inputs
                .first()
                .map(|input| input.sender_address.clone())
                .unwrap_or_else(|| "COINBASE".to_string()),
            output_count: tx.outputs.len(),
            total_output_amount: tx.get_total_output_amount(),
            estimated_size_bytes: tx.estimated_size_bytes(),
            fee_rate_sat_per_kb: fee_reference_utxo
                .and_then(|utxo_set| tx.calculate_fee_rate_sat_per_kb(utxo_set)),
        })
        .collect();

    Ok(Json(entries))
}

async fn create_transaction(
    State(state): State<ApiState>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<CreateTransactionResponse>), ApiHttpError> {
    if !req.amount_coin.is_finite() || req.amount_coin <= 0.0 {
        return Err(ApiHttpError::bad_request(
            "amount_coin sıfırdan büyük olmalı",
        ));
    }

    let amount_satoshi = (req.amount_coin * 100_000_000.0).round();
    if amount_satoshi <= 0.0 {
        return Err(ApiHttpError::bad_request("amount_coin çok küçük"));
    }

    let mut network = state
        .network
        .lock()
        .map_err(|_| ApiHttpError::internal("İşlem oluşturulamadı"))?;

    if req.sender_id >= network.node_count() || req.recipient_id >= network.node_count() {
        return Err(ApiHttpError::bad_request(
            "Geçersiz sender_id veya recipient_id",
        ));
    }

    let recipient_address = network.get_node_address(req.recipient_id);
    let Some(tx) =
        network.create_transaction(req.sender_id, &recipient_address, amount_satoshi as u64)
    else {
        return Err(ApiHttpError::bad_request("İşlem oluşturulamadı"));
    };

    if let Err(err) = network.save_to_disk(&state.state_path) {
        return Err(ApiHttpError::internal(format!(
            "State kaydedilemedi: {}",
            err
        )));
    }

    Ok((
        StatusCode::CREATED,
        Json(CreateTransactionResponse { tx_id: tx.id }),
    ))
}

async fn mine_block(
    State(state): State<ApiState>,
) -> Result<(StatusCode, Json<MineBlockResponse>), ApiHttpError> {
    let mut network = state
        .network
        .lock()
        .map_err(|_| ApiHttpError::internal("Madencilik çalıştırılamadı"))?;

    if network.current_val_id().is_none() {
        network.select_random_validator();
    }

    let Some(block) = network.mine_block() else {
        return Err(ApiHttpError::bad_request("Blok üretilemedi"));
    };

    if let Err(err) = network.save_to_disk(&state.state_path) {
        return Err(ApiHttpError::internal(format!(
            "State kaydedilemedi: {}",
            err
        )));
    }

    Ok((
        StatusCode::OK,
        Json(MineBlockResponse {
            index: block.index,
            hash: block.hash,
            transaction_count: block.transactions.len(),
        }),
    ))
}

struct ApiHttpError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiHttpError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiHttpError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(ApiError {
            code: self.code,
            message: self.message,
        });
        (self.status, body).into_response()
    }
}
