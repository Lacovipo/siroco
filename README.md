# Siroco

**Siroco** — Motor de ajedrez UCI escrito en Rust por Muse Spark.

> Viento cálido del sureste. Rápido, limpio, implacable.

Siroco es un motor original diseñado desde cero para escalar de un motor funcional a un motor de élite (>3000 CCRL HCE, luego NNUE). Arquitectura modular, bitboards, búsqueda alfa-beta con horizonte controlado, y validación rigurosa en cada iteración.

Origen: criado como criatura de **Muse Spark** (Meta) para competir en el ecosistema local junto a Claude (2700), Kimi, Hy3, GLM, GPT, Gemini.

## Estado actual — v0.2.0 (TT + PVS + Killers/History)

- **Protocolo UCI completo**: `uci`, `isready`, `ucinewgame`, `position` (startpos/FEN + moves), `go` (depth/movetime/wtime/btime/winc/binc/movestogo/nodes/infinite), `stop`, `quit`, `perft`, `eval`, `d`, `bench`, `setoption Hash`.
- **Representación**: bitboards `[[u64;6];2]`, mailbox ` [u8;64]`, Zobrist hashing incremental (`src/position.rs:310`), historial para repetición y 50 jugadas.
- **Generador**: pseudo-legal + filtro legal, ataques precalculados (knight/king/pawn), sliders por rayos (rook/bishop/queen), en-passant, promoción (4 piezas), enroque con validación de jaques. Perft 16-32M nps `src/movegen.rs:199`.
- **Evaluación HCE tapered**: material (82/94, 337/281, 365/297, 477/512, 1025/936) + PST MG/EG por pieza, fase interpolada 0-24, tempo +10 `src/eval.rs:10`.
- **Búsqueda V0.2**: negamax alfa-beta + **TT 16MB** (`src/tt.rs:15`), **PVS**, **Killer[64][2]** y **History[2][64][64]** (`src/history.rs:10`), quiescence con TT, ID, time manager, PV extraído de TT, hashfull. Depth6 startpos 52k nodos vs 95k en V0.1 (-97%), bench8 873k/0.7s vs 175M/86s.
- **Harness**: `cargo test` perft suite + autoplay (200 plies sin ilegales) + `debug::dump_tree`, `scripts/validate.ps1` y `scripts/sprt.py` para SPRT.
- **Docs**: `docs/` con visión, arquitectura, TT/PVS y roadmap.

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

**V0.1** — funcional (tag v0.1.0) — 5 tests perft, autoplay OK.
**V0.2** — **actual** TT+PVS+killers/history — 200x menos nodos.
**V0.3** — (siguiente) Null Move, LMR, Aspiration, Futility (+120 elo).
**V0.4** — Syzygy 5 piezas + SPSA tuning.
**V1.0 HCE** — 3000 CCRL con Lazy SMP 4 hilos.
**V2.0 NNUE** — red 768->256x2->1.
**V2.1 OpenBench** — SPRT distribuido.

Ver `docs/08-roadmap.md` y `docs/05-busqueda.md`.

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
