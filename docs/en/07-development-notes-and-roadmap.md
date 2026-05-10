# Development Notes and Roadmap

## Implemented Improvements

Recent iterations have introduced:

- modular test structure,
- stronger transaction validation (signature and ownership checks),
- mempool conflict prevention through outpoint tracking,
- fee accounting and coinbase reward cap validation,
- typed OutPoint-based UTXO identity model,
- wallet state reconstruction during chain synchronization without identity reset.

## Short-Term Recommendations

1. Resolve existing compiler warnings and tighten lint policy.
2. Extend full-chain validation with fee and coinbase constraints at every block.
3. Clarify mempool ordering policy (for example, fee-prioritized selection).

## Mid-Term Recommendations

1. Improve network model realism and message scheduling behavior.
2. Externalize runtime parameters into CLI/configuration files.
3. Expand reorganization and synchronization test coverage.

## Long-Term Recommendations

1. Introduce persistent state storage and deterministic reload.
2. Add API and visualization layers for observability.
3. Support broader consensus experimentation scenarios.
