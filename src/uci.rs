use crate::history::{HistoryTable, KillerTable};
use crate::perft::{perft, perft_divide};
use crate::position::Position;
use crate::search::{search_with_tt, SearchLimits};
use crate::tt::TranspositionTable;
use crate::types::*;

use std::io::{self, BufRead};

const NAME: &str = "Siroco";
const VERSION: &str = "0.2.0";
const AUTHOR: &str = "Muse Spark";

pub fn run() {
    let mut pos = Position::new();
    let mut tt = TranspositionTable::new(16);
    let mut history = HistoryTable::new();
    let mut killers = KillerTable::new();
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = stdin.lock().read_line(&mut line).unwrap();
        if bytes == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let tokens: Vec<String> = trimmed.split_whitespace().map(|s| s.to_string()).collect();
        if tokens.is_empty() {
            continue;
        }
        let cmd = tokens[0].as_str();
        match cmd {
            "uci" => {
                println!("id name {} {}", NAME, VERSION);
                println!("id author {}", AUTHOR);
                println!("option name Hash type spin default 16 min 1 max 1024");
                println!("option name Threads type spin default 1 min 1 max 1");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                pos = Position::new();
                tt.clear();
                history.clear();
                killers.clear();
            }
            "position" => {
                if tokens.len() < 2 {
                    continue;
                }
                let mut idx = 1;
                let mut fen_str = String::new();
                if tokens[idx] == "startpos" {
                    fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
                    idx += 1;
                } else if tokens[idx] == "fen" {
                    idx += 1;
                    let mut parts = Vec::new();
                    while idx < tokens.len() && tokens[idx] != "moves" {
                        parts.push(tokens[idx].clone());
                        idx += 1;
                    }
                    fen_str = parts.join(" ");
                } else {
                    continue;
                }
                match Position::from_fen(&fen_str) {
                    Ok(new_pos) => pos = new_pos,
                    Err(e) => {
                        eprintln!("info string FEN error: {}", e);
                        continue;
                    }
                }
                if idx < tokens.len() && tokens[idx] == "moves" {
                    idx += 1;
                    while idx < tokens.len() {
                        let mv_str = &tokens[idx];
                        if let Some(mv) = Move::from_uci(mv_str) {
                            let mut list = MoveList::new();
                            crate::movegen::generate_legal(&mut pos, &mut list);
                            let mut found = false;
                            for &legal in list.as_slice() {
                                if legal.from_sq() == mv.from_sq()
                                    && legal.to_sq() == mv.to_sq()
                                    && legal.promotion() == mv.promotion()
                                {
                                    pos.make_move(legal);
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                eprintln!("info string illegal move in position: {}", mv_str);
                            }
                        } else {
                            eprintln!("info string bad move format: {}", mv_str);
                        }
                        idx += 1;
                    }
                }
            }
            "go" => {
                let params = tokens[1..].to_vec();
                let limits = SearchLimits::from_go_params(&params);
                let (best, _score) = search_with_tt(&mut pos, limits, &mut tt, &mut history, &mut killers);
                if best.is_null() {
                    println!("bestmove 0000");
                } else {
                    println!("bestmove {}", best.to_uci());
                }
            }
            "stop" => {}
            "quit" | "exit" => break,
            "d" | "display" | "board" => {
                print_board(&pos);
                println!("Hashfull: {} TT hits: {} stores: {}", tt.hashfull(), tt.hits, tt.stores);
            }
            "eval" => {
                let score = crate::eval::evaluate(&pos);
                println!("info string eval {} (side to move)", score);
                println!("FEN: {}", pos.to_fen());
            }
            "perft" => {
                if tokens.len() < 2 {
                    println!("info string usage: perft <depth>");
                    continue;
                }
                let depth: u32 = tokens[1].parse().unwrap_or(0);
                let mut total = 0u64;
                let start = std::time::Instant::now();
                if depth == 0 {
                    println!("info string perft 0 = 1");
                } else {
                    let divide = perft_divide(&mut pos, depth);
                    for (uci, nodes) in &divide {
                        println!("{}: {}", uci, nodes);
                        total += nodes;
                    }
                    let elapsed = start.elapsed().as_millis();
                    println!("Nodes: {}", total);
                    println!("Time: {} ms", elapsed);
                    if elapsed > 0 {
                        println!("NPS: {}", total * 1000 / elapsed as u64);
                    }
                    let check = perft(&mut pos, depth);
                    assert_eq!(check, total);
                }
            }
            "fen" => {
                println!("{}", pos.to_fen());
            }
            "setoption" => {
                // setoption name <id> value <x>
                let mut name = String::new();
                let mut value = String::new();
                let mut reading_name = false;
                let mut reading_value = false;
                for tok in &tokens[1..] {
                    if tok == "name" {
                        reading_name = true;
                        reading_value = false;
                        continue;
                    } else if tok == "value" {
                        reading_name = false;
                        reading_value = true;
                        continue;
                    }
                    if reading_name {
                        if !name.is_empty() {
                            name.push(' ');
                        }
                        name.push_str(tok);
                    } else if reading_value {
                        if !value.is_empty() {
                            value.push(' ');
                        }
                        value.push_str(tok);
                    }
                }
                let name_lc = name.to_ascii_lowercase();
                if name_lc == "hash" {
                    if let Ok(mb) = value.parse::<usize>() {
                        let clamped = mb.clamp(1, 1024);
                        tt.resize(clamped);
                        println!("info string Hash set to {} MB ({} entries)", clamped, tt.entries.len());
                    }
                } else if name_lc == "threads" {
                    println!("info string Threads option ignored (single thread)");
                }
            }
            "bench" => {
                let depth = if tokens.len() > 1 {
                    tokens[1].parse::<u32>().unwrap_or(6)
                } else {
                    12
                };
                bench(depth, &mut tt, &mut history, &mut killers);
            }
            _ => {
                eprintln!("info string unknown command: {}", cmd);
            }
        }
        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }
}

fn print_board(pos: &Position) {
    println!(" +---+---+---+---+---+---+---+---+");
    for rank in (0..8).rev() {
        print!("{} |", rank + 1);
        for file in 0..8 {
            let sq = make_square(file, rank);
            let p = pos.board[sq as usize];
            let c = if p == NO_PIECE {
                ' '
            } else {
                let pt = piece_type(p);
                let mut ch = pt.to_char();
                if piece_color(p) == Color::White {
                    ch = ch.to_ascii_uppercase();
                }
                ch
            };
            print!(" {} |", c);
        }
        println!();
        println!(" +---+---+---+---+---+---+---+---+");
    }
    println!("   a   b   c   d   e   f   g   h");
    println!("FEN: {}", pos.to_fen());
    println!("Side: {}", if pos.side_to_move == Color::White { "white" } else { "black" });
    println!("Castling: {}", pos.castling);
    println!("En passant: {}", if pos.en_passant == NO_SQUARE { "-".to_string() } else { square_name(pos.en_passant) });
    println!("Hash: {:016x}", pos.hash);
    println!("Halfmove: {} Fullmove: {}", pos.halfmove, pos.fullmove);
}

fn bench(depth: u32, tt: &mut TranspositionTable, history: &mut HistoryTable, killers: &mut KillerTable) {
    let start = std::time::Instant::now();
    let mut pos = Position::new();
    let limits = SearchLimits {
        depth: Some(depth),
        ..Default::default()
    };
    tt.clear();
    let (best, score) = search_with_tt(&mut pos, limits, tt, history, killers);
    let elapsed = start.elapsed().as_millis();
    println!("bench depth {} best {} score {} time {} ms nodes hashfull {}", depth, best.to_uci(), score, elapsed, tt.hashfull());
}
