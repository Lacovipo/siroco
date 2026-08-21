# Roadmap Siroco

## V0.1 — Baseline funcional (tag v0.1.0)
- UCI completo, bitboards, perft 32M nps, HCE simple, negamax+qsearch, ID.

## V0.2 — TT + PVS + Killers/History (actual, tag v0.2.0)
- TT 16MB (1M entradas) con mate adjust, PVS, Killer[64][2], History[2][64][64].
- Bench depth8 175M→0.8M nodos, mismo score.

## V0.3 — Null + LMR + Aspiration + Futility (actual, v0.3.0)
- **Null Move:** `R=2/3` si `depth>=3 && !in_check && has_non_pawn_material`, `make_null/unmake_null`.
- **LMR:** `moves_searched>=4 && depth>=3 && quiet && !killer` → `depth-1 - reduction` (1-2 según history) + re-search si `>alpha`.
- **Aspiration:** `prev ±25` desde depth4, dobla a 50/100/INF si fail.
- **Futility:** `depth==1 && stand_pat+120 <= alpha && quiet` → skip.
- **Medido:** -95% nodos bench8 vs V0.2, score idéntico.

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
