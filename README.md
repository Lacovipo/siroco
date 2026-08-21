# Siroco

**Siroco** — Motor de ajedrez UCI escrito en Rust por Muse Spark.

> Viento cálido del sureste. Rápido, limpio, implacable.

Siroco es un motor original diseñado desde cero para escalar de un motor funcional a un motor de élite (>3000 CCRL HCE, luego NNUE). Arquitectura modular, bitboards, búsqueda alfa-beta con horizonte controlado, y validación rigurosa en cada iteración.

Origen: criado como criatura de **Muse Spark** (Meta) para competir en el ecosistema local junto a Claude (2700), Kimi, Hy3, GLM, GPT, Gemini.

## Estado actual — v0.5.0 (Syzygy + SPSA)

- **Protocolo UCI completo**: `uci`, `isready`, `ucinewgame`, `position` (startpos/FEN + moves), `go` (depth/movetime/wtime/btime/winc/binc/movestogo/nodes/infinite), `stop`, `quit`, `perft`, `eval`, `d`, `bench`, `setoption Hash`, `setoption SyzygyPath`.
- **Representación**: bitboards `[[u64;6];2]`, mailbox ` [u8;64]`, Zobrist hashing incremental (`src/position.rs:310`), historial para repetición y 50 jugadas, `make_null`/`unmake_null` (`src/position.rs:730`).
- **Generador**: pseudo-legal + filtro legal, **excluye captura de rey** (`src/movegen.rs:194`), Perft 16-32M nps.
- **Evaluación HCE V0.5**: V0.4 + **SPSA tune** aislado -12→-14, pareja 20→22 (`src/eval.rs:230`). Tapered MG/EG.
- **Búsqueda V0.5**: V0.4 + **Syzygy stub 5 piezas** WDL side-to-move (`src/syzygy.rs:1`), probe en `negamax`/`qsearch` si `ply>0` y `enabled`. KQK win en 2 nodos con Syzygy.
- **Harness**: `cargo test` 8 tests (5 perft +3 syzygy) + autoplay + `scripts/tune.py` SPSA + `scripts/validate.ps1`.
- **Docs**: `docs/` 11 ficheros. Ver `docs/11-v0.5.md`.

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
**V0.4** — Evaluación enriquecida + Extensión jaque
**V0.5** — **actual** Syzygy + SPSA
**V1.0 HCE** — (siguiente) Lazy SMP + Syzygy real + SPSA masivo
**V2.0 NNUE** — red 768->256x2->1
**V2.1 OpenBench** — SPRT distribuido

Ver `docs/08-roadmap.md`, `docs/05-busqueda.md`, `docs/11-v0.5.md`.

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
