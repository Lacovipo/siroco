# Evaluación HCE

File: `src/eval.rs:1`

## Diseño Tapered

`MG_VALUES = [82,337,365,477,1025,0]` / `EG_VALUES = [94,281,297,512,936,0]` + PST.

PST MG/EG por pieza (64 casillas, índice `rank*8+file`, `rank0=a1`):
- Peón MG/EG `PST_PAWN_MG:20` → avance central, EG más empuje.
- Caballo MG/EG `PST_KNIGHT_MG:35` → centralización.
- Alfil MG/EG `PST_BISHOP_MG:57` → diagonales.
- Torre MG/EG, Dama MG/EG, Rey MG (enrocado) / EG (centralizar) `PST_KING_EG:155`.

Fórmula:
```
phase = sum(PHASE_WEIGHTS[pt]) // N=1 B=1 R=2 Q=4, max 24
mg = Σ sign*(MG_VALUES[pt] + PST_MG[pt][sq_mirror])
eg = Σ sign*(EG_VALUES[pt] + PST_EG[pt][sq_mirror])
score_white = (mg*phase + eg*(24-phase))/24
score_stm = side_sign*score_white + 10 // tempo
```
`sq_mirror = sq ^ 56` si negra (`eval.rs:120`).

## Propósito V0.1-V0.2

Muy simple, sin movilidad ni estructura peones, pero con PST cerca de PeSTO simplificado. Suficiente para ~1800-2000 elo y base para tuning SPSA (V1.3). Sin NNUE aún; cuando se alcance 3000 CCRL se entrenará red `pytorch-nnue` y se sustituirá `evaluate()` por incremental.

## Test

`uci eval` imprime score white. No hay tests unitarios de eval aún; se valida indirectamente vía `autoplay` (partidas no caóticas, jaques controlados).
