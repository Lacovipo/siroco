# TT, PVS, Killers & History (V0.2)

## Transposition Table `src/tt.rs:1`

- **Estructura:** `TTEntry { key:u64, best_move:Move(u32), score:i32, depth:u8, flag:u8, age:u8 }` 16 bytes. `TranspositionTable { entries:Vec<TTEntry>, mask, age, hits, stores }`.
- **Tamaño:** `new(mb)` calcula `num_entries = next_pow2(mb*1M/16)`. Default 16MB → 1M entradas, 1MB → 65k, 1024MB → 67M. `size_mb()` reporta.
- **Indexado:** `hash as usize & mask` (bits bajos). Colisión 1/2^64 despreciable; se verifica `key == hash`.
- **Almacenado:** `store(hash, move, score, depth, flag, ply)` ajusta mate `±ply`, reemplaza si `key==0 || age!=entry.age || depth>=entry.depth || flag==EXACT`.
- **Sondeo:** `probe(hash) -> Option<TTEntry>` incrementa `hits`. `retrieve_with_correction` desajusta mate.
- **Edad:** `new_search()` incrementa `age`; `clear()` resetea y `age++`. `hashfull:87` cuenta `age==current` por mil.
- **Tests:** `tt_store_probe` y `tt_mate_adjust` en `tt.rs:120`.

## PVS (Principal Variation Search)

Implementado en `negamax` (`search.rs:450`). Primera jugada ventana completa, resto null-window `(-alpha-1, -alpha)`. Si falla alto (`>alpha && <beta`) re-búsqueda completa. Reduce ~50% nodos vs alfa-beta puro con buen orden.

## Killers `src/history.rs:10`

`KillerTable { killers: [[Move;2];64] }`
- Solo quiets no capturas/promos.
- `store(ply, mv)`: desplaza `[1]=[0], [0]=mv` si no duplicado.
- Orden: `+8000` primer killer, `+7000` segundo.

## History `src/history.rs:35`

`HistoryTable { table: [[[i32;64];64];2] }` // color, from, to
- `update(color, mv, depth, bonus)` con `bonus = depth*depth` si buena, `-depth*depth` si mala, fórmula `entry += bonus - entry*|bonus|/16384` clamp `±16384` (Stockfish-like).
- `score(color, mv)` devuelve `history/32` para encajar con killers/capturas.
- En `beta cutoff` se premia la jugada y penaliza `quiets_searched` previas.

## Integración UCI `src/uci.rs:15`

- `run()` mantiene `tt:TranspositionTable`, `history:HistoryTable`, `killers:KillerTable` persistentes.
- `ucinewgame` → `clear()` los tres + nueva posición.
- `setoption name Hash value <mb>` → `tt.resize(mb)` y `info string`.
- `go` → `search_with_tt(&mut pos, limits, &mut tt, &mut history, &mut killers)` → PV extraído de TT.
- `d` muestra `hashfull` y stats.

## Validación

- Perft no afectado (no usa TT).
- Autoplay 5 partidas depth2-3 sigue sin ilegales, nodos por depth3 ~1k vs 6k antes.
- Bench depth8 nodos 200x menos demuestra TT funciona; score idéntico `cp 10` y bestmove `b1c3` vs V0.1.

## Próximos pasos

- V1.2 añadirá `LMR` (usa history), `null move`, `aspiration windows` (±25 cp alrededor del score previo) y `futility`.
- `History` se usará para LMR.
