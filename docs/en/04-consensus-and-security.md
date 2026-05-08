# Consensus and Security

## Consensus Approach

The simulation applies a majority-oriented chain acceptance strategy. Nodes verify incoming blocks and accept chains that satisfy validation constraints and chain selection rules.

## Block Validation Criteria

A candidate block is accepted only if:

1. Block index sequencing is correct.
2. `previous_hash` matches the local chain tip hash.
3. Recomputed block hash matches the provided hash.
4. Hash satisfies Proof of Work difficulty.
5. Merkle root is consistent with included transactions.
6. The first transaction is coinbase and no additional coinbase transaction exists.
7. Coinbase output total does not exceed `base_reward + total_fees`.

## Transaction-Level Security

Security at transaction level is enforced by:

- ECDSA signature validation,
- ownership verification through address derivation,
- duplicate input prevention within a transaction,
- mempool-level double-spend conflict checks.

## Manipulation Scenarios

The simulation allows controlled observation of:

- hash manipulation attempts,
- block content tampering,
- local chain divergence and recovery behavior.

These scenarios support validation of detection and correction mechanisms.
