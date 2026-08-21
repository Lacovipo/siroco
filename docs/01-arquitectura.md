# Arquitectura Siroco

```
siroco/
  Cargo.toml          # Rust edition 2021, profile release lto=thin
  src/
    lib.rs            # declaración módulos
    main.rs           # bin que llama uci::run
    types.rs          # Color, PieceType, Move, MoveList, Square
    position.rs       # Board + Zobrist + FEN + make/unmake + draw
    movegen.rs        # ataques + generate_legal
    eval.rs           # HCE tapered
    search.rs         # negamax + qsearch + ID + TT + PVS + killers/history
    tt.rs             # TranspositionTable
    history.rs        # KillerTable + HistoryTable
    perft.rs          # contador divide
    uci.rs            # loop UCI persistente
    debug.rs          # dump_tree
  tests/
    perft_extended.rs
    autoplay.rs
  scripts/
    validate.ps1
    sprt.py
  docs/
  Release/            # binarios versionados por usuario
```

## Flujo de datos

`UCI (stdin) → uci.rs:12 run() → Position::from_fen → search_with_tt(pos, limits, &mut tt, &mut history, &mut killers) → bestmove`

- **Estado persistente UCI:** `Position`, `TranspositionTable (16MB default)`, `HistoryTable`, `KillerTable`. `ucinewgame` limpia los 4. `setoption name Hash` recalcula `tt.resize(mb)` (`src/uci.rs:110`).
- **Estado efímero búsqueda:** `Searcher` contiene referencias mutuas a los 4 + límites de tiempo.

## Dependencias

Cero dependencias externas. Solo `std`. Esto garantiza portabilidad y `cargo build --release` reproducible.

## Compilación

- `cargo build --release` → `target/release/siroco.exe` (≈ 700KB strip)
- `cargo test` → perft (5 tests lib + 2 perft_extended) + autoplay (3 tests)
- `cargo bench` no usado; `perft` y `bench` vía UCI son benchmarks.

## Versionado

Git tags `v0.1.0` → `v0.2.0` (actual). `Cargo.toml:3 version` sincronizado con `uci.rs:8 VERSION`.
