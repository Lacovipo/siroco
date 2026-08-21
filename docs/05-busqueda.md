# Búsqueda

File: `src/search.rs:1` (680 líneas en V0.2)

## V0.1 — Baseline

- Negamax alfa-beta puro, sin TT
- Quiescence solo capturas/promos, evasión si jaque
- ID (iterative deepening) con `time/30 + inc*0.75` `soft/hard` 50ms overhead
- Orden MVV-LVA
- Mate distance pruning, 50 jugadas/repetición → 0
- Nodos depth4 startpos 95k

## V0.2 — TT + PVS + Killers/History

### Búsqueda principal

`negamax:330` firma `fn negamax(searcher, depth, alpha, beta, ply) -> i32`:

1. **TT probe** (`src/tt.rs:60`): `hash & mask` → `TTEntry { key, best_move, score, depth, flag, age }`. Flag `EXACT/LOWER/UPPER`. Mate scores ajustados `±ply`. Si `depth_entry >= depth` y flag compatible → cutoff. En ply 0 actualiza `best_move` antes de retornar.

2. **Generación legal** y `is_empty` → mate/stalemate.

3. **Ordenación** `score_move:100`:
   - TT move +20000
   - Promoción +900 + pieza, captura `10*victim - attacker +1000`
   - Killer `8000/7000` (`src/history.rs:10 KillerTable`)
   - History `history[color][from][to]/32` (`src/history.rs:35 HistoryTable`)

4. **PVS**:
```rust
if moves_searched==0 { score = -negamax(depth-1, -beta, -alpha) }
else {
  score = -negamax(depth-1, -alpha-1, -alpha);
  if score > alpha && score < beta { score = -negamax(depth-1, -beta, -alpha) }
}
```
5. **Actualización killers/history** en `beta cutoff` solo si `!is_capture && !promo`: `killers.store(ply, mv)` y `history.update_quiet(depth*depth)` + penaliza quiets previas.

6. **TT store** con flag según `best_score vs original_alpha / beta`.

### Quiescence

`qsearch:227` depth 0: stand_pat `evaluate`, si `>=beta` return, genera capturas/promos (o todo si jaque), ordena con `score_move`, recurre.

### Iterative Deepening

`search_with_tt:520` loop `depth 1..=max_depth`, llama `negamax`, extrae PV vía `extract_pv:470` (sigue TT), imprime `info depth score cp/mate nodes nps time hashfull pv ...`, respeta `soft_limit` y `hard_limit`, TT `new_search()` incrementa `age`.

**Wrapper** `search:510` crea TT 16MB efímero para tests/bench; UCI usa el persistente.

## V0.3 — Null + LMR + Aspiration + Futility (`src/search.rs:375`)

- **Null:** `if can_null && depth>=3 && !in_check && has_non_pawn_material && beta<MATE` → `make_null(); -negamax(depth-1-R, -beta, -beta+1);` R=2/3.
- **Futility:** `depth==1 && !in_check && !cap && !killer && stand_pat+120 <= alpha && moves_searched>0` → skip quiet.
- **LMR:** `depth>=3 && moves_searched>=4 && quiet && !killer` → `reduction=1 (+1 si hist<-1000 && depth>=6)` → `new_depth=depth-1-reduction` null-window, re-search si `>alpha`.
- **Aspiration:** depth≥4 `alpha=prev-25 beta=prev+25`, loop dobla a 50/100/INF hasta dentro.

## Métricas

- depth4 startpos V0.1 95k → V0.2 2.5k (-97%) → **V0.3 980 (-98.9% vs V0.1)**
- depth6 V0.2 52k → **V0.3 6.7k (-87%)**
- depth8 bench V0.1 175M/86s → V0.2 873k/0.7s → **V0.3 37k/24ms (-95% vs V0.2)**
- PV idéntico, score idéntico, `mate 1` sigue `f7g7`/`h5f7`.
- hashfull depth8 23‰ (menos por poda).
