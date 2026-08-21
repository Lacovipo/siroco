# Representación del Tablero

## Elección: Bitboards

`Position:82` mantiene:
- `board: [u8;64]` mailbox para acceso O(1) `piece_at:290`
- `bb: [[u64;6];2]` 12 bitboards (índice `color*6 + piece_type`)
- `occupied: [u64;2]`, `occupied_all: u64`
- `side_to_move: Color`, `castling: u8` (bits `CASTLE_WK=1,WQ=2,BK=4,BQ=8` `src/types.rs:153`), `en_passant: u8 (64=none)`, `halfmove, fullmove`, `hash: u64`, `history: Vec<State>`, `hash_history: Vec<u64>`

**Por qué bitboards y no 0x88:** el objetivo es NNUE incremental. Con bitboards, actualizar acumulador tras `make_move` es `xor` de piezas; con mailbox habría que escanear. Coste de generación es similar, pero escalabilidad es máxima.

## Zobrist

`src/position.rs:10 Zobrist { piece_keys[12][64], side_key, castling_keys[16], en_passant_keys[8] }` generada vía `splitmix64` con semilla fija `0x1234...`. `hash` incremental en `make_move:376` / `unmake_move:532` con `debug_assert_eq!(hash, compute_hash())`.

`compute_hash:310` recalcula desde cero para validación.

## FEN

`from_fen:114` parsea 6 campos, rellena `bb` y `board`, computa hash. `to_fen:210` inverso para debug y perft.

## Make/Unmake

`make_move:336` (86 líneas) maneja:
- Captura normal y `en_passant` (`cap_sq = to ∓ 8`)
- Enroque (roque `h1→f1` etc.)
- Promoción (reemplaza `Pawn→Queen` etc.)
- Derechos de enroque (`from==A1 → clear WQ` etc.)
- `en_passant` solo si doble empuje 16
- `halfmove` reset si peón/captura
- `fullmove++` si negra movió
- Actualiza `bb`, `board`, `occupied` vía `recompute_occupied:299`, `hash_history`.

`unmake_move:532` revierte simétricamente usando `State { castling, en_passant, halfmove, captured, hash }` guardado en `history`. Probado con 119M nodos perft sin asserts.

## Detección de tablas

- `is_draw:620` → 50 jugadas `halfmove>=100`, `is_insufficient_material:636` (Kk, KBk, KNk, KBKB mismo color), `is_threefold:654`
- `is_repetition:671` usada en búsqueda para `ply>0` → score 0
