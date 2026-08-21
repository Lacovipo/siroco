# Roadmap Siroco

## V0.1 — Baseline funcional (tag v0.1.0)
- UCI completo, bitboards, perft 32M nps, HCE simple, negamax+qsearch, ID.

## V0.2 — TT + PVS + Killers/History (actual, tag v0.2.0)
- TT 16MB (1M entradas) con mate adjust, PVS, Killer[64][2], History[2][64][64].
- Bench depth8 175M→0.8M nodos, mismo score.

## V0.3 — Poda/Reducción (siguiente fase propuesta)
- **Null Move Pruning:** `R=3` si `!in_check && depth>=3 && eval>=beta` → `make_null` + `depth-R`.
- **LMR:** para `moves_searched>3 && depth>=3 && !capture && !promo && !in_check` → `depth-1` null-window, re-search si `>alpha`.
- **Aspiration Windows:** ID con `window=25` alrededor de `prev_score`, ampliar a `50,100,INF` si falla.
- **Futility:** si `depth==1 && !in_check && eval+120+... <= alpha` → podar quiets.
- **Est. elo:** +120-150 sobre V0.2 (medido vía SPRT 200g `10+0.1`).

## V0.4 — Syzygy y Tuning
- Syzygy 5 piezas (decisión del autor: implement antes de NNUE para finales). `TB probe` en root y `qsearch` si `halfmove==0` y material <=5.
- SPSA tuning de `PST` y `MG/EG` valores con 10k partidas.

## V1.0 HCE — 3000 CCRL
- Todo anterior + `Hash` ya funcional + `Threads` Lazy SMP (4 hilos).
- Objetivo validado vs Claude 2700 en suite local.

## V2.0 NNUE
- Red `pytorch-nnue` entrenada con datos de Siroco HCE + Leela.
- `eval::evaluate` → `nnue::evaluate` con actualización incremental tras `make/unmake` (usa `bb`).
- Mantiene HCE como fallback.
- Objetivo >3400.

## V2.1 OpenBench
- Integración OpenBench/SPRT distribuido para tuning continuo.

## Decisiones pendientes del autor (usuario delega)

- Momento Syzygy: propuesto V0.4 (antes de NNUE, aporta 15-20 elo baratos).
- NNUE arquitectura: `(768->256x2->1)` clásico, 16-bit.
