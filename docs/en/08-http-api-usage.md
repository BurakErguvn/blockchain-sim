# HTTP API Usage Guide

This document describes how to use the HTTP API provided by the `api_server` binary.

## 1) Start the API Server

```bash
cargo run --bin api_server
```

Default bind address:

- `http://0.0.0.0:3000`

Startup behavior:

1. The server first attempts to load persisted state (`./data/network_state.json`).
2. If no state is available, it bootstraps a default network (node setup + genesis).

## 2) Endpoint Overview

### 2.1 Health and Network

- `GET /health`
- `GET /network/state`

### 2.2 Nodes and Chain

- `GET /nodes`
- `GET /nodes/{id}`
- `GET /nodes/{id}/blockchain`

### 2.3 Mempool

- `GET /mempool`

### 2.4 Write Operations

- `POST /transactions`
- `POST /mine`

Write endpoints persist state after successful execution.

## 3) Request Examples

### 3.1 Health Check

```bash
curl http://127.0.0.1:3000/health
```

### 3.2 Network Summary

```bash
curl http://127.0.0.1:3000/network/state
```

### 3.3 List Nodes

```bash
curl http://127.0.0.1:3000/nodes
```

### 3.4 Node Details

```bash
curl http://127.0.0.1:3000/nodes/0
```

### 3.5 Show Blockchain

```bash
curl http://127.0.0.1:3000/nodes/0/blockchain
```

### 3.6 Show Mempool

```bash
curl http://127.0.0.1:3000/mempool
```

### 3.7 Create a Transaction

```bash
curl -X POST http://127.0.0.1:3000/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "sender_id": 0,
    "recipient_id": 1,
    "amount_coin": 1.25
  }'
```

### 3.8 Mine One Block

```bash
curl -X POST http://127.0.0.1:3000/mine
```

## 4) Error Format

API errors follow a consistent JSON response model:

```json
{
  "code": "bad_request",
  "message": "Invalid sender_id or recipient_id"
}
```

Common error codes:

- `bad_request`
- `not_found`
- `internal_error`

## 5) Operational Notes

- API and CLI can share the same state file; for safer operations, prefer a single active writer.
- For production-like environments, recommended next steps are:
  - CORS restrictions,
  - authentication/API keys,
  - rate limiting.
