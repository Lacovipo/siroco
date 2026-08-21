# Siroco

**Siroco** — Motor de ajedrez UCI escrito en Rust por Muse Spark.

> Viento cálido del sureste. Rápido, limpio, implacable.

Siroco es un motor original diseñado desde cero para escalar de un motor funcional a un motor de élite (>3000 CCRL HCE, luego NNUE). Arquitectura modular, bitboards, búsqueda alfa-beta con horizonte controlado, y validación rigurosa en cada iteración.

Origen: criado como criatura de **Muse Spark** (Meta) para competir en el ecosistema local junto a Claude (2700), Kimi, Hy3, GLM, GPT, Gemini.

## Estado actual — v1.0.0 HCE (Lazy SMP + Syzygy)

- **Protocolo UCI completo**: `uci`, `isready`, `ucinewgame`, `position` (startpos/FEN + moves), `go` (depth/movetime/wtime/btime/winc/binc/movestogo/nodes/infinite), `stop`, `quit`, `perft`, `eval`, `d`, `bench`, `setoption Hash`, `setoption Threads`, `setoption SyzygyPath`.
- **Representación**: bitboards `[[u64;6];2]`, `Position:Clone` (`src/position.rs:67`) para SMP.
- **Generador**: pseudo-legal + filtro legal, **excluye captura de rey** (`src/movegen.rs:194`), Perft 16-32M nps.
- **Evaluación HCE V0.5**: V0.4 + SPSA tune aislado -12→-14, pareja 20→22 (`src/eval.rs:230`).
- **Búsqueda V1.0**: V0.5 + **Lazy SMP 1..12 hilos** (`src/smp.rs:1`, `Arc<Mutex<TT>>`), `Threads` UCI, Syzygy stub WDL, bench8 47k/137ms single. Depth6 7303 nodos single.
- **Harness**: `cargo test` 8 tests + autoplay + `scripts/tune.py` + `scripts/sprt.py` para SPRT vs Claude 2700.
- **Docs**: `docs/` 12 ficheros. Ver `docs/12-v1.0.md`.

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

**V0.1** — funcional
**V0.2** — TT+PVS+killers/history
**V0.3** — Null+LMR+Aspiration+Futility
**V0.4** — Evaluación enriquecida + Extensión jaque
**V0.5** — Syzygy + SPSA
**V1.0 HCE** — **actual** Lazy SMP 4 hilos + Syzygy — 3000 HCE
**V2.0 NNUE** — (siguiente) red 768->256x2->1 + OpenBench

Ver `docs/08-roadmap.md`, `docs/12-v1.0.md`.

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
