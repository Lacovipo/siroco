use crate::eval::evaluate;
use crate::history::{HistoryTable, KillerTable};
use crate::movegen::{generate_legal, is_square_attacked};
use crate::position::Position;
use crate::tt::{TranspositionTable, TT_EXACT, TT_LOWER, TT_UPPER};
use crate::types::*;

use std::time::{Duration, Instant};

pub const MATE: i32 = 30000;
pub const INF: i32 = 31000;
const MAX_PLY: usize = 64;

#[derive(Debug, Clone, Default)]
pub struct SearchLimits {
    pub depth: Option<u32>,
    pub movetime: Option<u64>,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u32>,
    pub nodes: Option<u64>,
    pub infinite: bool,
}

impl SearchLimits {
    pub fn from_go_params(params: &[String]) -> Self {
        let mut limits = SearchLimits::default();
        let mut i = 0;
        while i < params.len() {
            match params[i].as_str() {
                "depth" => {
                    if i + 1 < params.len() {
                        limits.depth = params[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "movetime" => {
                    if i + 1 < params.len() {
                        limits.movetime = params[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "wtime" => {
                    if i + 1 < params.len() {
                        limits.wtime = params[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "btime" => {
                    if i + 1 < params.len() {
                        limits.btime = params[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "winc" => {
                    if i + 1 < params.len() {
                        limits.winc = params[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "binc" => {
                    if i + 1 < params.len() {
                        limits.binc = params[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "movestogo" => {
                    if i + 1 < params.len() {
                        limits.movestogo = params[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "nodes" => {
                    if i + 1 < params.len() {
                        limits.nodes = params[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "infinite" => limits.infinite = true,
                _ => {}
            }
            i += 1;
        }
        limits
    }
}

struct Searcher<'a> {
    pos: &'a mut Position,
    tt: &'a mut TranspositionTable,
    history: &'a mut HistoryTable,
    killers: &'a mut KillerTable,
    limits: SearchLimits,
    start: Instant,
    soft_limit: Option<Duration>,
    hard_limit: Option<Duration>,
    nodes: u64,
    best_move: Move,
    best_score: i32,
    stop: bool,
}

impl<'a> Searcher<'a> {
    fn should_stop(&self) -> bool {
        if self.stop {
            return true;
        }
        if let Some(nodes_limit) = self.limits.nodes {
            if self.nodes >= nodes_limit {
                return true;
            }
        }
        if let Some(hard) = self.hard_limit {
            if self.start.elapsed() >= hard {
                return true;
            }
        }
        false
    }

    fn time_exceeded_soft(&self) -> bool {
        if let Some(soft) = self.soft_limit {
            self.start.elapsed() >= soft
        } else {
            false
        }
    }

    #[inline]
    fn is_capture(&self, mv: Move) -> bool {
        let to = mv.to_sq();
        let moving = self.pos.board[mv.from_sq() as usize];
        if moving == NO_PIECE {
            return false;
        }
        let is_ep = piece_type(moving) == PieceType::Pawn && to == self.pos.en_passant;
        if is_ep {
            return true;
        }
        self.pos.board[to as usize] != NO_PIECE
    }

    fn score_move(&self, mv: Move, tt_move: Move, ply: usize) -> i32 {
        if mv.0 == tt_move.0 && !tt_move.is_null() {
            return 20000;
        }
        let from = mv.from_sq();
        let to = mv.to_sq();
        let moving = self.pos.board[from as usize];
        let captured = self.pos.board[to as usize];
        let is_ep = moving != NO_PIECE
            && piece_type(moving) == PieceType::Pawn
            && to == self.pos.en_passant;
        let victim = if is_ep {
            make_piece(self.pos.side_to_move.opposite(), PieceType::Pawn)
        } else {
            captured
        };
        let mut score = 0;
        if let Some(promo) = mv.promotion() {
            score += 900 + match promo {
                PieceType::Queen => 400,
                PieceType::Rook => 200,
                PieceType::Bishop => 100,
                PieceType::Knight => 100,
                _ => 0,
            };
            if victim != NO_PIECE {
                let victim_val = match piece_type(victim) {
                    PieceType::Pawn => 100,
                    PieceType::Knight => 320,
                    PieceType::Bishop => 330,
                    PieceType::Rook => 500,
                    PieceType::Queen => 900,
                    PieceType::King => 20000,
                };
                score += 10 * victim_val;
            }
            return score + 2000;
        }
        if victim != NO_PIECE {
            let victim_val = match piece_type(victim) {
                PieceType::Pawn => 100,
                PieceType::Knight => 320,
                PieceType::Bishop => 330,
                PieceType::Rook => 500,
                PieceType::Queen => 900,
                PieceType::King => 20000,
            };
            let attacker_val = if moving != NO_PIECE {
                match piece_type(moving) {
                    PieceType::Pawn => 100,
                    PieceType::Knight => 320,
                    PieceType::Bishop => 330,
                    PieceType::Rook => 500,
                    PieceType::Queen => 900,
                    PieceType::King => 20000,
                }
            } else {
                0
            };
            score += 10 * victim_val - attacker_val + 1000;
            return score;
        }
        let killer_score = self.killers.score_killer(ply, mv);
        if killer_score != 0 {
            return killer_score;
        }
        let hist = self.history.score(self.pos.side_to_move as usize, mv);
        score += hist / 32;
        score
    }
}

fn is_in_check(pos: &Position) -> bool {
    let king_sq = pos.king_square(pos.side_to_move);
    is_square_attacked(pos, king_sq, pos.side_to_move.opposite())
}

fn qsearch(searcher: &mut Searcher, mut alpha: i32, beta: i32, ply: usize) -> i32 {
    if searcher.should_stop() {
        return alpha;
    }
    searcher.nodes += 1;
    if ply >= MAX_PLY {
        return evaluate(searcher.pos);
    }

    if searcher.pos.is_draw() || searcher.pos.is_repetition() {
        return 0;
    }

    let hash = searcher.pos.hash;
    if let Some(entry) = searcher.tt.probe(hash) {
        let (score, _, _, flag) = searcher.tt.retrieve_with_correction(entry, ply as i32);
        if flag == TT_EXACT {
            return score;
        }
        if flag == TT_LOWER && score >= beta {
            return score;
        }
        if flag == TT_UPPER && score <= alpha {
            return score;
        }
    }

    let stand_pat = evaluate(searcher.pos);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let in_check = is_in_check(searcher.pos);

    let mut list = MoveList::new();
    if in_check {
        generate_legal(searcher.pos, &mut list);
    } else {
        let mut pseudo = MoveList::new();
        crate::movegen::generate_pseudo_legal(searcher.pos, &mut pseudo);
        for &mv in pseudo.as_slice() {
            let to = mv.to_sq();
            let is_capture = searcher.pos.board[to as usize] != NO_PIECE || to == searcher.pos.en_passant;
            if is_capture || mv.is_promotion() {
                let us = searcher.pos.side_to_move;
                searcher.pos.make_move(mv);
                let king_sq = searcher.pos.king_square(us);
                let illegal = is_square_attacked(searcher.pos, king_sq, searcher.pos.side_to_move);
                searcher.pos.unmake_move(mv);
                if !illegal {
                    list.push(mv);
                }
            }
        }
    }

    let tt_move = searcher
        .tt
        .probe(hash)
        .map(|e| e.best_move)
        .unwrap_or(Move::NULL);

    let mut scored: Vec<(Move, i32)> = list
        .as_slice()
        .iter()
        .map(|&m| (m, searcher.score_move(m, tt_move, ply)))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    let mut best_score = stand_pat;
    let mut best_move_q = Move::NULL;
    let original_alpha = alpha;

    for (mv, _) in scored {
        searcher.pos.make_move(mv);
        let score = -qsearch(searcher, -beta, -alpha, ply + 1);
        searcher.pos.unmake_move(mv);
        if searcher.should_stop() {
            return alpha;
        }
        if score >= beta {
            searcher.tt.store(hash, mv, beta, 0, TT_LOWER, ply as i32);
            return beta;
        }
        if score > best_score {
            best_score = score;
            best_move_q = mv;
        }
        if score > alpha {
            alpha = score;
        }
    }

    let flag = if best_score <= original_alpha {
        TT_UPPER
    } else if best_score >= beta {
        TT_LOWER
    } else {
        TT_EXACT
    };
    if !best_move_q.is_null() || flag != TT_EXACT {
        searcher
            .tt
            .store(hash, best_move_q, best_score, 0, flag, ply as i32);
    }

    alpha
}

fn negamax(
    searcher: &mut Searcher,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    ply: usize,
    can_null: bool,
) -> i32 {
    if searcher.should_stop() {
        return alpha;
    }
    if depth == 0 {
        return qsearch(searcher, alpha, beta, ply);
    }

    searcher.nodes += 1;

    if ply > 0 && (searcher.pos.is_draw() || searcher.pos.is_repetition()) {
        return 0;
    }

    let mate_alpha = -MATE + ply as i32;
    let mate_beta = MATE - ply as i32 - 1;
    if mate_alpha > alpha {
        alpha = mate_alpha;
        if alpha >= beta {
            return alpha;
        }
    }
    if mate_beta < beta {
        if mate_beta <= alpha {
            return mate_beta;
        }
    }

    let in_check = is_in_check(searcher.pos);
    let hash = searcher.pos.hash;
    let mut tt_move = Move::NULL;
    let mut tt_score = 0;
    let mut tt_flag = 0;
    let mut tt_depth = 0;
    let mut tt_hit = false;
    if let Some(entry) = searcher.tt.probe(hash) {
        let (score, mv, d, flag) = searcher.tt.retrieve_with_correction(entry, ply as i32);
        tt_move = mv;
        tt_score = score;
        tt_flag = flag;
        tt_depth = d;
        tt_hit = true;
        if tt_depth >= depth as u8 {
            if tt_flag == TT_EXACT {
                if ply == 0 && !tt_move.is_null() {
                    searcher.best_move = tt_move;
                }
                return tt_score;
            }
            if tt_flag == TT_LOWER && tt_score >= beta {
                if ply == 0 && !tt_move.is_null() {
                    searcher.best_move = tt_move;
                }
                return tt_score;
            }
            if tt_flag == TT_UPPER && tt_score <= alpha {
                return tt_score;
            }
        }
    }

    // Null move pruning
    if can_null
        && depth >= 3
        && !in_check
        && searcher.pos.has_non_pawn_material(searcher.pos.side_to_move)
        && ply > 0
        && beta < MATE - 100
        && beta > -MATE + 100
    {
        let r: i32 = if depth > 6 { 3 } else { 2 };
        searcher.pos.make_null();
        let null_score = -negamax(searcher, depth - 1 - r, -beta, -beta + 1, ply + 1, false);
        searcher.pos.unmake_null();
        if searcher.should_stop() {
            return alpha;
        }
        if null_score >= beta {
            return beta;
        }
    }

    // Futility pruning setup for shallow depths
    let stand_pat = if !in_check && depth <= 2 {
        evaluate(searcher.pos)
    } else {
        0
    };

    let mut list = MoveList::new();
    generate_legal(searcher.pos, &mut list);
    if list.is_empty() {
        if is_square_attacked(
            searcher.pos,
            searcher.pos.king_square(searcher.pos.side_to_move),
            searcher.pos.side_to_move.opposite(),
        ) {
            return -MATE + ply as i32;
        } else {
            return 0;
        }
    }

    let mut scored: Vec<(Move, i32)> = list
        .as_slice()
        .iter()
        .map(|&m| (m, searcher.score_move(m, tt_move, ply)))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    let mut best_score = -INF;
    let mut best_move = Move::NULL;
    let original_alpha = alpha;
    let mut moves_searched = 0;
    let mut quiets_searched: Vec<Move> = Vec::new();

    for (mv, _) in scored {
        let is_cap = searcher.is_capture(mv) || mv.is_promotion();
        let is_killer = searcher.killers.is_killer(ply, mv);

        // Futility pruning: depth 1, not in check, quiet, not killer, and stand_pat + margin <= alpha
        if depth == 1
            && !in_check
            && !is_cap
            && !is_killer
            && moves_searched > 0
            && stand_pat + 120 <= alpha
            && best_score > -MATE + 100
        {
            continue;
        }

        let mut new_depth = depth - 1;
        let mut is_reduced = false;
        let mut reduction = 0;

        // LMR: Late Move Reduction for quiets after first few moves
        // (no reduction if gives check — check after make)
        if depth >= 3
            && moves_searched >= 4
            && !is_cap
            && !in_check
            && !is_killer
        {
            let hist = searcher.history.score(searcher.pos.side_to_move as usize, mv);
            let base_reduction = if depth >= 6 && moves_searched >= 10 { 2 } else { 1 };
            reduction = if hist < -1000 { base_reduction + 1 } else { base_reduction };
            reduction = reduction.min(depth - 1);
            if reduction > 0 {
                new_depth = depth - 1 - reduction;
                is_reduced = true;
            }
        }

        searcher.pos.make_move(mv);
        // Check extension: if move gives check, extend 1 ply
        let gives_check = is_in_check(searcher.pos);
        let mut effective_depth = new_depth;
        if gives_check {
            effective_depth += 1;
            // if we reduced and gives check, undo reduction
            if is_reduced {
                is_reduced = false;
                effective_depth = depth; // actually depth (since new_depth was depth-1 - red, +1 => depth - red)
                // for simplicity, set to depth-1 if gave check (no reduction)
                effective_depth = depth - 1 + 1; // = depth
                // but cap to depth
                if effective_depth > depth {
                    effective_depth = depth;
                }
            } else {
                // already not reduced, just extension
                if effective_depth > depth {
                    effective_depth = depth;
                }
            }
        }
        let score;
        if moves_searched == 0 {
            score = -negamax(searcher, effective_depth, -beta, -alpha, ply + 1, true);
        } else {
            // PVS null window, with possible LMR
            let mut null_score = -negamax(searcher, effective_depth, -alpha - 1, -alpha, ply + 1, true);
            if is_reduced && null_score > alpha {
                // re-search at full depth if reduced search failed high
                null_score = -negamax(searcher, depth - 1, -alpha - 1, -alpha, ply + 1, true);
            }
            if null_score > alpha && null_score < beta {
                score = -negamax(searcher, depth - 1, -beta, -alpha, ply + 1, true);
            } else {
                score = null_score;
            }
        }
        searcher.pos.unmake_move(mv);

        if searcher.should_stop() {
            return best_score;
        }

        if score > best_score {
            best_score = score;
            best_move = mv;
            if ply == 0 {
                searcher.best_move = mv;
                searcher.best_score = score;
            }
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            if !is_cap {
                searcher.killers.store(ply, mv);
                searcher.history.update_quiet(searcher.pos.side_to_move as usize, mv, depth, true);
                for &q in &quiets_searched {
                    searcher.history.update_quiet(searcher.pos.side_to_move as usize, q, depth, false);
                }
            }
            break;
        }
        if !is_cap {
            quiets_searched.push(mv);
        }
        moves_searched += 1;
    }

    let flag = if best_score <= original_alpha {
        TT_UPPER
    } else if best_score >= beta {
        TT_LOWER
    } else {
        TT_EXACT
    };
    searcher
        .tt
        .store(hash, best_move, best_score, depth as u8, flag, ply as i32);

    best_score
}

fn extract_pv(pos: &mut Position, tt: &TranspositionTable) -> Vec<Move> {
    let mut pv = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for _ in 0..32 {
        let hash = pos.hash;
        let idx = (hash as usize) & tt.mask;
        let entry = tt.entries[idx];
        if entry.key != hash || entry.best_move.is_null() {
            break;
        }
        let mv = entry.best_move;
        let mut list = MoveList::new();
        generate_legal(pos, &mut list);
        if !list.as_slice().contains(&mv) {
            break;
        }
        if !seen.insert(hash) {
            break;
        }
        pv.push(mv);
        pos.make_move(mv);
        if pv.len() >= 16 {
            break;
        }
    }
    for &mv in pv.iter().rev() {
        pos.unmake_move(mv);
    }
    pv
}

// Public wrapper that creates ephemeral TT/history/killers (for tests/bench)
pub fn search(pos: &mut Position, limits: SearchLimits) -> (Move, i32) {
    let mut tt = TranspositionTable::new(16);
    let mut history = HistoryTable::new();
    let mut killers = KillerTable::new();
    search_with_tt(pos, limits, &mut tt, &mut history, &mut killers)
}

pub fn search_with_tt(
    pos: &mut Position,
    limits: SearchLimits,
    tt: &mut TranspositionTable,
    history: &mut HistoryTable,
    killers: &mut KillerTable,
) -> (Move, i32) {
    let start = Instant::now();
    let mut soft_limit: Option<Duration> = None;
    let mut hard_limit: Option<Duration> = None;

    if let Some(mt) = limits.movetime {
        soft_limit = Some(Duration::from_millis(mt));
        hard_limit = Some(Duration::from_millis(mt));
    } else if limits.wtime.is_some() || limits.btime.is_some() {
        let is_white = pos.side_to_move == Color::White;
        let my_time = if is_white { limits.wtime } else { limits.btime };
        let my_inc = if is_white { limits.winc } else { limits.binc };
        if let Some(time) = my_time {
            let inc = my_inc.unwrap_or(0);
            let movestogo = limits.movestogo.unwrap_or(30) as u64;
            let overhead: u64 = 50;
            let mut soft = time / movestogo + inc * 3 / 4;
            if soft > time / 3 {
                soft = time / 3;
            }
            if soft > time.saturating_sub(overhead) {
                soft = time.saturating_sub(overhead);
            }
            let mut hard = soft * 3 / 2;
            if hard > time.saturating_sub(overhead) {
                hard = time.saturating_sub(overhead);
            }
            if soft < 10 && time > 10 {
                soft = 10;
            }
            if hard < 10 && time > 10 {
                hard = 10;
            }
            soft_limit = Some(Duration::from_millis(soft));
            hard_limit = Some(Duration::from_millis(hard));
        }
    }

    let max_depth = limits.depth.unwrap_or(64) as i32;
    let mut best_move = Move::NULL;
    let mut best_score = 0;

    tt.new_search();

    let mut searcher = Searcher {
        pos,
        tt,
        history,
        killers,
        limits: limits.clone(),
        start,
        soft_limit,
        hard_limit,
        nodes: 0,
        best_move: Move::NULL,
        best_score: 0,
        stop: false,
    };

    let mut root_list = MoveList::new();
    generate_legal(searcher.pos, &mut root_list);
    if root_list.is_empty() {
        return (Move::NULL, 0);
    }

    // Aspiration windows
    let mut prev_score = 0;
    let mut delta = 25;

    for depth in 1..=max_depth {
        let mut alpha = -INF;
        let mut beta = INF;
        if depth >= 4 {
            alpha = (prev_score - delta).max(-INF);
            beta = (prev_score + delta).min(INF);
        }

        let mut score;
        loop {
            score = negamax(&mut searcher, depth, alpha, beta, 0, true);
            if searcher.should_stop() {
                break;
            }
            if score <= alpha {
                // fail low
                delta *= 2;
                if delta > 500 {
                    alpha = -INF;
                    beta = INF;
                    score = negamax(&mut searcher, depth, alpha, beta, 0, true);
                    break;
                }
                alpha = (prev_score - delta).max(-INF);
                beta = (prev_score + delta).min(INF);
                // also need to keep beta wider? For fail low we expand alpha down
                // continue loop to re-search
                if score > alpha && score < beta {
                    break;
                }
                // else continue
            } else if score >= beta {
                delta *= 2;
                if delta > 500 {
                    alpha = -INF;
                    beta = INF;
                    score = negamax(&mut searcher, depth, alpha, beta, 0, true);
                    break;
                }
                alpha = (prev_score - delta).max(-INF);
                beta = (prev_score + delta).min(INF);
                if score > alpha && score < beta {
                    break;
                }
            } else {
                break;
            }
            if delta > 200 {
                // fallback to full window to avoid infinite loops
                alpha = -INF;
                beta = INF;
            }
        }

        if searcher.should_stop() {
            // if we stopped during search, don't update best
            if searcher.best_move.is_null() {
                break;
            }
            best_move = searcher.best_move;
            best_score = prev_score; // keep old
            break;
        }
        best_move = searcher.best_move;
        best_score = score;
        prev_score = best_score;
        delta = 25; // reset for next depth

        let elapsed = searcher.start.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        let nps = if elapsed_ms > 0 {
            searcher.nodes * 1000 / elapsed_ms
        } else {
            searcher.nodes
        };

        let score_str = if best_score.abs() > MATE - 100 {
            let mate_in = if best_score > 0 {
                (MATE - best_score + 1) / 2
            } else {
                -(MATE + best_score) / 2
            };
            format!("mate {}", mate_in)
        } else {
            format!("cp {}", best_score)
        };

        let pv_moves = extract_pv(searcher.pos, &*searcher.tt);
        let pv_str = if pv_moves.is_empty() {
            best_move.to_uci()
        } else {
            pv_moves.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" ")
        };
        let hashfull = searcher.tt.hashfull();

        println!(
            "info depth {} score {} nodes {} nps {} time {} hashfull {} pv {}",
            depth, score_str, searcher.nodes, nps, elapsed_ms, hashfull, pv_str
        );

        if searcher.time_exceeded_soft() && depth >= 1 {
            if !searcher.limits.infinite && searcher.limits.movetime.is_none() {
                if searcher.soft_limit.is_some() {
                    break;
                }
            }
        }
        if best_score.abs() > MATE - 100 {
            break;
        }
        if depth as u64 >= 64 {
            break;
        }
    }

    if best_move.is_null() {
        let mut list = MoveList::new();
        generate_legal(searcher.pos, &mut list);
        if !list.is_empty() {
            best_move = list.moves[0];
        }
    }

    (best_move, best_score)
}
