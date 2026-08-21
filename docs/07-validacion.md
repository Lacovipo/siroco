# Validación y Harness

## Batería permanente

Cada iteración debe pasar `scripts/validate.ps1:1` (PowerShell, `Continue` para warnings):

1. `cargo test -- --nocapture` — 5 tests lib (3 perft + 2 TT)
2. `cargo test --test perft_extended` — 7 FENs, profundidad 3-4, incluye `n1n5/PPPk4/...` promo 24/496
3. `cargo test --test autoplay` — 5 partidas 150-200 plies depth2-3, verifica `hash == compute_hash` tras cada `make/unmake`, 0 ilegales, `is_draw` correcto.
4. `cargo build --release` + `perft 5` → `4865609` + `bench 8` + `uci go depth4`

**Resultados V0.2 (191s → 92s tras TT):**
- perft básico 17.8s, perft_extended 48s, autoplay 26s, bench8 0.7s (vs 86s V0.1)
- `perft 5` 16-32M nps, `bench 8` 873k nodos hashfull 488‰, `uci depth6` 52k nodos

## Debug

- `src/debug.rs:6 dump_tree(pos, depth, "tree.log")` vuelca árbol `ply move alpha beta fen eval`.
- UCI `d` imprime tablero + `hashfull`/`hits`/`stores`, `eval` y `fen`.
- Asserts `debug_assert_eq!(hash, compute_hash())` en `make/unmake` activos en debug.

## SPRT para parches

`scripts/sprt.py:1`:
- Si `cutechess-cli` presente y `baseline != candidate`: `cutechess-cli -engine cmd=... tc=10+0.1 -games 200 -repeat -concurrency 4`, parsea `Score: W-L-D`, calcula `elo = -400 log10(1/score-1)` y `SE`, SPRT `elo0=0 elo1=15 α=β=0.05`.
- Si no hay cutechess: proxy ejecuta `cargo test --test autoplay` y reporta no-regresión.

**Uso:**
```bash
python scripts/sprt.py --games 200 --tc 10+0.1 --baseline Release/Siroco\ 0.1.exe --candidate target/release/siroco.exe
# En PowerShell el espacio necesita `"Release/Siroco 0.1.exe"`
```

Para el ecosistema local (Claude 2700 etc.) se recomienda `tc 8+0.08` y 500-1000 partidas para discriminar 10±15 elo.

## Release

Usuario pone binarios en `Release/` (ej. `Siroco 0.1.exe`, `Siroco 0.2.exe` tras este parche). Script no toca `Release/` automáticamente; el agente copia tras `cargo build --release` si se pide.
