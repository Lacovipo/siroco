# Documentación Siroco

Índice técnico versionado.

- `00-vision.md` — filosofía y métricas
- `01-arquitectura.md` — estructura de carpetas y flujo UCI
- `02-representacion.md` — bitboards, Zobrist, make/unmake, tablas
- `03-movegen.md` — generador pseudo-legal → legal, perft
- `04-evaluacion.md` — HCE tapered y PST (+ estructura peones en V0.4)
- `05-busqueda.md` — V0.1 vs V0.2 vs V0.3 vs V0.4, PVS, Null, LMR
- `06-tt-pvs-killers.md` — detalle TT, PVS, Killers/History, UCI
- `07-validacion.md` — harness, bench, SPRT
- `08-roadmap.md` — V1.0 HCE y V2.0 NNUE
- `09-v0.3.md` — Null, LMR, Aspiration, Futility (V0.3)
- `10-v0.4.md` — Evaluación enriquecida + extensión jaque (V0.4)

Cada iteración añade un `.md` o actualiza existentes. Los fuentes son la referencia primaria (`src/*:línea`).
