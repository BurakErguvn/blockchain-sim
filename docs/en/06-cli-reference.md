# Command-Line Interface Reference

The application provides an interactive command-line interface after startup.

## Commands

### `bakiye <node_id>`

Displays the balance of the selected node.

Example:

```text
bakiye 2
```

### `transfer <sender_id> <receiver_id> <amount>`

Creates a transfer from sender node to receiver node.

Example:

```text
transfer 0 3 1.25
```

### `durum`

Prints a summary of network state.

### `blockchain <node_id>`

Prints blockchain summary for the selected node.

### `mempool`

Lists pending transactions currently stored in mempool.

### `çıkış` (aliases: `exit`, `quit`)

Terminates the simulation.

## Error Cases

The interface reports:

- invalid node identifiers,
- missing command parameters,
- invalid numeric formats,
- insufficient balance conditions.
