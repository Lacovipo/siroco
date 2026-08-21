# Siroco

**Siroco** — Motor de ajedrez UCI escrito en Rust por Muse Spark.

> Viento cálido del sureste. Rápido, limpio, implacable.

Siroco es un motor original diseñado desde cero para escalar de un motor funcional a un motor de élite (>3000 CCRL HCE, luego NNUE). Arquitectura modular, bitboards, búsqueda alfa-beta con horizonte controlado, y validación rigurosa en cada iteración.

Origen: criado como criatura de **Muse Spark** (Meta) para competir en el ecosistema local junto a Claude (2700), Kimi, Hy3, GLM, GPT, Gemini.

## Estado actual — v0.1.0 (Esqueleto funcional → Motor jugable)

- **Protocolo UCI completo**: `uci`, `isready`, `ucinewgame`, `position` (startpos/FEN + moves), `go` (depth/movetime/wtime/btime/winc/binc/movestogo/nodes/infinite), `stop`, `quit`, `perft`, `eval`, `d`, `bench`.
- **Representación**: bitboards `[[u64;6];2]`, mailbox ` [u8;64]`, Zobrist hashing incremental, historial para repetición y 50 jugadas.
- **Generador**: pseudo-legal + filtro legal, ataques precalculados (knight/king/pawn), sliders por rayos (rook/bishop/queen), en-passant, promoción (4 piezas), enroque con validación de jaques.
- **Perft validado**: `startpos` 20/400/8902/197281/4865609, Kiwipete, Position3, etc. 17-29M nps en release.
- **Evaluación HCE tapered**: material (82/94, 337/281, 365/297, 477/512, 1025/936) + PST MG/EG por pieza, fase interpolada 0-24, tempo +10.
- **Búsqueda**: negamax alfa-beta, quiescence solo capturas/promociones (evasión de jaque con todo legal), iterative deepening, time manager (`time/30 + inc*0.75`, soft/hard), MVV-LVA ordering, mate distance pruning, detección de tablas (repetición, 50 jugadas, material insuficiente).
- **Harness**: `cargo test` perft suite + autoplay (200 plies sin ilegales), `debug::dump_tree`, `scripts/validate.ps1`.

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

**V1.0** (actual) — funcional, ~1800-2000 elo estimado.
**V1.1** — TT (Zobrist, depth replacement), killer/history, PVS.
**V1.2** — Null move, LMR, futility, aspiration.
**V1.3** — Tuning SPSA de PST + valores, Syzygy (decisión del autor).
**V2.0** — NNUE incremental (pytorch-nnue), threads Lazy SMP, Hash.
**V2.1** — OpenBench / cutechess SPRT para validar cada patch.

Diseño preparado para NNUE: bitboards ya permiten actualización incremental eficiente.

## Arquitectura

```
src/
  types.rs     — Color, PieceType, Square, Move, MoveList
  position.rs  — Board, make/unmake, FEN, Zobrist, draw
  movegen.rs   — generador legal, attacks
  eval.rs      — HCE tapered
  search.rs    — negamax + qsearch + ID + time
  perft.rs     — contador y tests
  uci.rs       — loop UCI
  debug.rs     — dump_tree
tests/
  perft_extended.rs
  autoplay.rs
scripts/
  validate.ps1 — harness completo
```

Cada módulo con API fina para poder sacarlo del contexto una vez estable.

## Licencia

MIT — uso personal, gratuito, código abierto. Ver `LICENSE`.

## Autor

Creado por **Muse Spark** (opencode/muse-spark-1.2-contributor-free) a petición del usuario. Ésta es su criatura.

Compite con respeto contra Claude, Kimi, Hy3, GLM, GPT, Gemini.
