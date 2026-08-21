# Roadmap Siroco

## V0.1 — Baseline funcional (tag v0.1.0)
- UCI completo, bitboards, perft 32M nps, HCE simple, negamax+qsearch, ID.

## V0.2 — TT + PVS + Killers/History (tag v0.2.0)
- TT 16MB con mate adjust, PVS, Killer[64][2], History[2][64][64].

## V0.3 — Null + LMR + Aspiration + Futility (tag v0.3.0)
- **Null Move:** `R=2/3`, **LMR:** `depth-1 - reduction`, **Aspiration:** `prev ±25`, **Futility:** `+120`.

## V0.4 — Evaluación Enriquecida + Extensión Jaque (tag v0.4.0)
- **Eval:** peones aisl/dobl/pasados, pareja alfiles, movilidad, torre abierta, escudo rey.
- **Búsqueda:** extensión `+1` si `gives_check`.

## V0.5 — Syzygy + SPSA (actual, v0.5.0)
- **Syzygy stub** 5 piezas WDL side-to-move, `SyzygyPath` UCI, probe en `negamax`/`qsearch` si `ply>0`.
- **SPSA** `scripts/tune.py` + ajuste aislado -12→-14, pareja 20→22.
- Medido perft idéntico, bench similar, `KQK` win en 2 nodos.

## V1.0 HCE — Próximo (siguiente)
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
