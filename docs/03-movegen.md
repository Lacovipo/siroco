# Generador de Movimientos

File: `src/movegen.rs:1`

## Precalculados

`Attacks { knight[64], king[64], pawn[2][64] }` en `init_attacks:10` via `OnceLock`. Para sliders, `bishop_attacks:70` y `rook_attacks:89` generan rayos al vuelo con loop `dr,df` y `occ` — exacto y sin magias por ahora (V0.2). V2 introducirá magics sin cambiar API `generate_legal`.

## Pseudo-legal

`generate_pseudo_legal:199` (120 líneas):
- Peones: simple + doble, capturas, `en_passant == pos.en_passant`, promoción 4 piezas (`Q,R,B,N`) tanto en empuje como captura, rank/file checks.
- Caballos/alfiles/torres/damas/rey vía `attacks & !own_occ` y `add_moves`.
- Enroque: solo si rey no en jaque `!is_square_attacked(pos, king_sq, them)`, casillas vacías y no atacadas (`f1,g1` etc.).

## Legal

`generate_legal:279` filtra pseudo haciendo `make_move` + `is_square_attacked(king, them)` y `unmake`. Coste ~2x pero perft sigue a 16-32M nps release.

## Qsearch

`generate_captures:408` no usado directo; `qsearch` en `search.rs:227` genera pseudo y filtra a capturas+promos, verificando legalidad igual que `generate_legal`.

## Ataque

`is_square_attacked:115` comprueba peones (`pawn_attacks(opposite)`), caballos, rey, sliders (`bishop/rook_attacks(sq, occ) & queens`). Usado en `make` validación y en búsqueda para `is_in_check`.

## Validación

Perft suite CPW:
- `src/perft.rs:48` startpos 8902/197281/4865609, Kiwipete 4085603, Position3 43238
- `tests/perft_extended.rs:5` 7 FENs + en_passant/promo tricky 24/496
- En `cargo test` 5 lib + 2 extended = 7 pos, 17M nps debug, 32M nps release.

Futuro: magias precomputadas para +30% nps, manteniendo `generate_legal` estable.
