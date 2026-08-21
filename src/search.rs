use crate::eval::evaluate;
use crate::movegen::{generate_legal, is_square_attacked};
use crate::position::Position;
use crate::types::*;

use std::time::{Duration, Instant};

pub const MATE: i32 = 30000;
pub const INF: i32 = 31000;
const MAX_PLY: i32 = 64;

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
    limits: SearchLimits,
    start: Instant,
    soft_limit: Option<Duration>,
    hard_limit: Option<Duration>,
    nodes: u64,
    best_move: Move,
    best_score: i32,
    stop: bool,
    pv: Vec<Move>,
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

    fn score_move(&self, mv: Move) -> i32 {
        let from = mv.from_sq();
        let to = mv.to_sq();
        let moving = self.pos.board[from as usize];
        let captured = self.pos.board[to as usize];
        // en passant
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
            // promotions are highly ordered
            score += 900 + match promo {
                PieceType::Queen => 400,
                PieceType::Rook => 200,
                PieceType::Bishop => 100,
                PieceType::Knight => 100,
                _ => 0,
            };
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
            score += 10 * victim_val - attacker_val;
            // bonus for capture
            score += 1000;
        }
        score
    }
}

fn is_in_check(pos: &Position) -> bool {
    let king_sq = pos.king_square(pos.side_to_move);
    is_square_attacked(pos, king_sq, pos.side_to_move.opposite())
}

fn qsearch(searcher: &mut Searcher, mut alpha: i32, beta: i32, ply: i32) -> i32 {
    if searcher.should_stop() {
        return alpha;
    }
    searcher.nodes += 1;
    if ply >= MAX_PLY {
        return evaluate(searcher.pos);
    }

    // draw check
    if searcher.pos.is_draw() || searcher.pos.is_repetition() {
        return 0;
    }

    let stand_pat = evaluate(searcher.pos);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    // if in check, we must generate all legal moves to avoid missing mate evasion
    let in_check = is_in_check(searcher.pos);

    let mut list = MoveList::new();
    if in_check {
        generate_legal(searcher.pos, &mut list);
        // in check, search all moves
    } else {
        // generate captures only
        let mut pseudo = MoveList::new();
        crate::movegen::generate_pseudo_legal(searcher.pos, &mut pseudo);
        // filter to captures + promos and legality
        for &mv in pseudo.as_slice() {
            let to = mv.to_sq();
            let is_capture = searcher.pos.board[to as usize] != NO_PIECE || to == searcher.pos.en_passant;
            if is_capture || mv.is_promotion() {
                // legality test: make and check if king still not in check
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

    // order captures
    let mut scored: Vec<(Move, i32)> = list.as_slice().iter().map(|&m| (m, searcher.score_move(m))).collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    for (mv, _) in scored {
        searcher.pos.make_move(mv);
        let score = -qsearch(searcher, -beta, -alpha, ply + 1);
        searcher.pos.unmake_move(mv);
        if searcher.should_stop() {
            return alpha;
        }
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

fn negamax(searcher: &mut Searcher, depth: i32, mut alpha: i32, beta: i32, ply: i32) -> i32 {
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

    // mate distance pruning
    let mate_alpha = -MATE + ply;
    let mate_beta = MATE - ply - 1;
    if mate_alpha > alpha {
        alpha = mate_alpha;
        if alpha >= beta {
            return alpha;
        }
    }
    if mate_beta < beta {
        // we must not overwrite beta mut? use local
        // if beta > mate_beta, we can lower beta
        // but caller expects beta, we just prune
        if mate_beta <= alpha {
            return mate_beta;
        }
    }

    let mut list = MoveList::new();
    generate_legal(searcher.pos, &mut list);
    if list.is_empty() {
        if is_square_attacked(searcher.pos, searcher.pos.king_square(searcher.pos.side_to_move), searcher.pos.side_to_move.opposite()) {
            return -MATE + ply;
        } else {
            return 0;
        }
    }

    // Move ordering at interior: score and sort
    let mut scored: Vec<(Move, i32)> = list.as_slice().iter().map(|&m| (m, searcher.score_move(m))).collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    let mut best_score = -INF;
    let mut moves_searched = 0;

    for (mv, _) in scored {
        searcher.pos.make_move(mv);
        // PVS can be added later; for now full window
        let score = -negamax(searcher, depth - 1, -beta, -alpha, ply + 1);
        searcher.pos.unmake_move(mv);

        if searcher.should_stop() {
            return best_score;
        }

        if score > best_score {
            best_score = score;
            if ply == 0 {
                searcher.best_move = mv;
                searcher.best_score = score;
            }
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
        moves_searched += 1;
    }

    best_score
}

pub fn search(pos: &mut Position, limits: SearchLimits) -> (Move, i32) {
    let start = Instant::now();
    // compute time limits
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
            // Basic allocation: time / movestogo + inc * 0.7
            // Ensure overhead 50ms
            let overhead: u64 = 50;
            let mut soft = time / movestogo + inc * 3 / 4;
            // clamp soft to time/3
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
            // minimum 10ms
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

    // iterative deepening
    let mut searcher = Searcher {
        pos,
        limits: limits.clone(),
        start,
        soft_limit,
        hard_limit,
        nodes: 0,
        best_move: Move::NULL,
        best_score: 0,
        stop: false,
        pv: Vec::new(),
    };

    // If only one legal move, we can return quickly but still search for score
    let mut root_list = MoveList::new();
    generate_legal(searcher.pos, &mut root_list);
    if root_list.is_empty() {
        return (Move::NULL, 0);
    }
    if root_list.len() == 1 {
        // still need to search to get score but we can just return quickly
        // but do iterative for info
    }

    let mut completed_depth = 0;
    for depth in 1..=max_depth {
        let score = negamax(&mut searcher, depth, -INF, INF, 0);
        if searcher.should_stop() {
            break;
        }
        best_move = searcher.best_move;
        best_score = score;
        completed_depth = depth;

        let elapsed = searcher.start.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        let nps = if elapsed_ms > 0 {
            searcher.nodes * 1000 / elapsed_ms
        } else {
            searcher.nodes
        };

        // UCI info
        // Score handling: mate
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

        println!(
            "info depth {} score {} nodes {} nps {} time {} pv {}",
            depth,
            score_str,
            searcher.nodes,
            nps,
            elapsed_ms,
            best_move.to_uci()
        );

        if searcher.time_exceeded_soft() && depth >= 1 {
            // if we have a move and soft limit exceeded, break
            // but only if not infinite
            if !limits.infinite && limits.movetime.is_none() {
                // check soft
                // For movetime we must reach depth
                // For incremental, we break
                if searcher.soft_limit.is_some() {
                    break;
                }
            }
        }

        // if mate found, we can stop early
        if best_score.abs() > MATE - 100 {
            break;
        }

        if depth as u64 >= 64 {
            break;
        }
    }

    // If search was stopped before any depth completed, fallback to first legal move
    if best_move.is_null() {
        // pick first move
        let mut list = MoveList::new();
        generate_legal(searcher.pos, &mut list);
        if !list.is_empty() {
            best_move = list.moves[0];
        }
    }

    // Use completed info for final bestmove, but searcher already holds final best
    (best_move, best_score)
}
