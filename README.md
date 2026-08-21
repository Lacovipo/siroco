# Siroco

**Siroco** — Motor de ajedrez UCI escrito en Rust por Muse Spark.

> Viento cálido del sureste. Rápido, limpio, implacable.

Siroco es un motor original diseñado desde cero para escalar de un motor funcional a un motor de élite (>3000 CCRL HCE, luego NNUE). Arquitectura modular, bitboards, búsqueda alfa-beta con horizonte controlado, y validación rigurosa en cada iteración.

Origen: criado como criatura de **Muse Spark** (Meta) para competir en el ecosistema local junto a Claude (2700), Kimi, Hy3, GLM, GPT, Gemini.

## Estado actual — v0.4.0 (Evaluación enriquecida + Extensión jaque)

- **Protocolo UCI completo**: `uci`, `isready`, `ucinewgame`, `position` (startpos/FEN + moves), `go` (depth/movetime/wtime/btime/winc/binc/movestogo/nodes/infinite), `stop`, `quit`, `perft`, `eval`, `d`, `bench`, `setoption Hash`.
- **Representación**: bitboards `[[u64;6];2]`, mailbox ` [u8;64]`, Zobrist hashing incremental (`src/position.rs:310`), historial para repetición y 50 jugadas, `make_null`/`unmake_null` (`src/position.rs:730`).
- **Generador**: pseudo-legal + filtro legal, ataques precalculados (knight/king/pawn), sliders por rayos (rook/bishop/queen), en-passant, promoción (4 piezas), enroque con validación de jaques. Perft 16-32M nps `src/movegen.rs:199`.
- **Evaluación HCE V0.4**: material+PST + **estructura peones (aislados/doblados/pasados)** + **pareja alfiles** + **movilidad N/B/R/Q** + **torre abierta** + **escudo rey** (`src/eval.rs:165`, `230-365`). Tapered MG/EG.
- **Búsqueda V0.4**: **Null R=2/3**, **LMR** quiet tardíos, **Futility 120**, **Aspiration 25**, **Extensión jaque +1** (`src/search.rs:496`) sobre TT+PVS+killers/history. Bench8 47k/137ms, kiwipete depth6 143k.
- **Harness**: `cargo test` perft suite + autoplay (200 plies sin ilegales) + `debug::dump_tree`, `scripts/validate.ps1` y `scripts/sprt.py`.
- **Docs**: `docs/` con visión, arquitectura, TT/PVS/LMR y roadmap. Ver `docs/10-v0.4.md`.

## Uso

```bash
cargo build --release
./target/release/siroco
```

GUI (Arena, Banksia, Cute Chess): añadir ejecutable como motor UCI.

Comandos manuales:
```
uci
isready
position startpos
go depth 10
go wtime 60000 btime 60000 winc 1000 binc 1000
perft 5
eval
d
quit
```

## Validación

```bash
# Batería completa
cargo test -- --nocapture
cargo test --test perft_extended -- --nocapture
cargo test --test autoplay -- --nocapture

# Perft rápido
echo "perft 5" | cargo run --release
# Bench
echo "bench 12" | cargo run --release

# Script validación (perft + autoplay + bench)
powershell -ExecutionPolicy Bypass -File scripts/validate.ps1
```

Salida esperada: todos perft exactos, 5 partidas autoplay sin ilegales, bench estable.

## Roadmap hacia 3000 CCRL

**V0.1** — funcional (tag v0.1.0)
**V0.2** — TT+PVS+killers/history
**V0.3** — Null+LMR+Aspiration+Futility
**V0.4** — **actual** Evaluación enriquecida + Extensión jaque
**V0.5** — (siguiente) Syzygy 5 piezas + SPSA tuning
**V1.0 HCE** — 3000 CCRL con Lazy SMP
**V2.0 NNUE** — red 768->256x2->1
**V2.1 OpenBench** — SPRT distribuido

Ver `docs/08-roadmap.md`, `docs/05-busqueda.md`, `docs/10-v0.4.md`.

## Arquitectura

```
src/
  types.rs     — Color, PieceType, Square, Move, MoveList
  position.rs  — Board, make/unmake, FEN, Zobrist, draw
  movegen.rs   — generador legal, attacks
  eval.rs      — HCE tapered
  search.rs    — negamax + qsearch + ID + time + TT + PVS
  tt.rs        — TranspositionTable
  history.rs   — KillerTable + HistoryTable
  perft.rs     — contador y tests
  uci.rs       — loop UCI persistente
  debug.rs     — dump_tree
tests/
  perft_extended.rs
  autoplay.rs
scripts/
  validate.ps1 — harness completo
  sprt.py      — SPRT
docs/          — documentación técnica versionada
```

Cada módulo con API fina para poder sacarlo del contexto una vez estable.

## Licencia

MIT — uso personal, gratuito, código abierto. Ver `LICENSE`.

## Autor

Creado por **Muse Spark** (opencode/muse-spark-1.2-contributor-free) a petición del usuario. Ésta es su criatura.

Compite con respeto contra Claude, Kimi, Hy3, GLM, GPT, Gemini.
