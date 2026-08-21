# Siroco — Visión y Filosofía

**Autor:** Muse Spark (Meta) — criatura del usuario
**Objetivo final:** >3000 CCRL HCE, luego NNUE >3400
**Competidores locales:** Claude 2700, Kimi/Hy3/GLM ~2550-2600, GPT/Gemini ~2400-2500

## Principios

1. **Modularidad radical.** Cada módulo (tablero, generación, evaluación, búsqueda, TT) con API fina, testeable aislado y re-emplazable. `src/types.rs:1` define el contrato; `src/position.rs:82` nunca importa `movegen`.
2. **Exactitud antes que velocidad.** Perft exacto en todas las posiciones CPW antes de acelerar. Debug hash `compute_hash:310` assert en `make/unmake`.
3. **Iteración con harness.** Cada parche debe pasar `scripts/validate.ps1:1` (perft + autoplay + bench) y, cuando se disponga de cutechess, SPRT `scripts/sprt.py:1` con `elo0=0 elo1=15`.
4. **Diseño para el futuro.** Bitboards `[[u64;6];2]` (`src/types.rs:310`) elegidos desde `v0.1` para que NNUE incremental sea un añadido, no una reescritura. TT dimensionable vía UCI `Hash`.
5. **Sin atajos.** Implementación original, sin copiar Stockfish. Inspiración conceptual de CPW, pero código propio.

## Métricas de éxito

- **V0.1** funcional: 29M nps perft, bestmove legal en cualquier posición, autoplay 200 plies sin ilegales.
- **V0.2** (actual): TT+PVS+killers/history, nodos depth6 startpos 52k vs 950k en V0.1 (-94%), PV completo vía TT.
- **V0.3 objetivo:** LMR + null-move + aspiration → profundidad media +2 en mismo tiempo, 2300-2500 elo estimado.
- **V1.0 HCE:** 3000 CCRL con Syzygy y tuning.

## Decisión de nombre

`Siroco` — viento del sur. Corto, UCI `id name Siroco 0.2.0` (`src/uci.rs:8`), sin "chess", libre en ecosistema, evocador de velocidad.
