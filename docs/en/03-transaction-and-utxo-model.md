# Transaction and UTXO Model

## Transaction Structure

A transaction consists of:

- **Inputs (`TxInput`)** that reference spendable outputs through `OutPoint`,
- **Outputs (`TxOutput`)** that define value and recipient address,
- a **transaction identifier** derived from transaction content.

## OutPoint-Based Identity

UTXO identity is represented by a typed structure:

- `OutPoint { txid, vout }`

Compared to string-concatenated identifiers, this model provides:

- improved type safety,
- elimination of parsing ambiguity,
- clearer key semantics for indexed lookups.

## UTXO Set Representation

Node-level and wallet-level UTXO sets are represented as:

- `HashMap<OutPoint, UTXO>`

This allows direct key-based retrieval for validation and state transitions.

## Signature and Ownership Validation

A transaction input is considered valid only if all of the following hold:

1. The referenced UTXO exists.
2. The public key in the input maps to the UTXO recipient address.
3. The signature verifies against the corresponding signing payload.
4. The same `OutPoint` is not repeated within the same transaction.
5. `total_output <= total_input`.

## Fee Model

For non-coinbase transactions:

`fee = total_input - total_output`

During block construction, transaction fees are accumulated and added to base block reward. Coinbase output must not exceed the sum of base reward and total fees.
