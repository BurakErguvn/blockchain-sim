# Ratatui Student Interface

The classic CLI remains available for automation and JSON output. Interactive classroom use can launch the full-screen Ratatui dashboard:

```bash
cargo run --bin sim_cli -- --profile classroom tui
```

The dashboard combines lab and task selection, guided instructions, chain movement, node balances, validator state, mempool contents, and an activity log.

## Keyboard

| Key | Action |
|-----|--------|
| `Tab`, `←`, `→` | Change panel focus |
| `↑`, `↓`, `j`, `k` | Move selection |
| `Enter`, `a` | Start a lab or apply the current task |
| `c` | Check the current task |
| `h` | Reveal the next hint |
| `m` | Mine one block |
| `r` | Reload state |
| `?` | Open help |
| `q`, `Esc`, `Ctrl-C` | Exit safely |

Future work includes a compact layout, mouse support, a branching fork graph, localization, themes, and live API event streams.
