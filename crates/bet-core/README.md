# bet-core

Pure game rules and virtual-points ledger for BET.

## Principles (Carmack × Bellard)

- **No I/O** — no filesystem, no sockets, no wall-clock RNG.
- **Deterministic** — same seed + inputs ⇒ same state hash.
- **Small surface** — games expose `apply` / tick; UI lives elsewhere.
- **Ledger separate** — stakes never mixed into pixel/sim code.

## Modules

| Module | Role |
|--------|------|
| `hangman` | Pure hangman rules |
| `tictactoe` | Pure board + optional seeded AI |
| `ledger` | Virtual points balances / stakes / settle |
| `hash` | Simple FNV-1a state fingerprints for goldens |

## Tests

```bash
cargo test -p bet-core
```
